extern crate num;
use num::PrimInt;

use std::ops::Add;

pub struct CritBit<K,V>( Option<CritBitNode<K,V>> ) where K: PrimInt;

pub enum CritBitNode<K,V> where K: PrimInt {
    Leaf ( K, V ),
    Internal ( (Option<Box<CritBitNode<K,V>>>, Option<Box<CritBitNode<K,V>>>), u32 ),
}

#[inline(always)]
fn bit_at<T: PrimInt>( value: &T, pos: &u32 ) -> bool {
    value.rotate_left(*pos).leading_zeros() == 0
}

impl<K,V> CritBit<K,V> where K: PrimInt {
    pub fn new() -> CritBit<K,V> {
        CritBit( None )
    }

    pub fn clear( &mut self ) {
        self.0 = None;
    }

    pub fn len( &self ) -> usize {
        self.0.iter().map(CritBitNode::len).fold(0, Add::add)
    }

    pub fn get( &self, key: &K ) -> Option<&V> {
        match &self.0 {
            &Some( ref node ) => node.get( key ),
            &None => None
        }
    }

    pub fn get_mut( &mut self, key: &K ) -> Option<&mut V> {
        match &mut self.0 {
            &mut Some( ref mut node ) => node.get_mut( key ),
            &mut None => None
        }
    }

    pub fn contains_key( &self, key: &K ) -> bool {
        self.get( key ).is_some()
    }

    pub fn insert( &mut self, key: K, value: V ) -> Option<V> {
        match &mut self.0 {
            &mut Some( ref mut node ) => node.insert( key, value ),
            x => { std::mem::replace( x, Some( CritBitNode::Leaf( key, value ) ) ); None }
        }
    }
}

impl<K: PrimInt, V> CritBitNode<K, V> {
    fn len( &self ) -> usize {
        match *self {
            CritBitNode::Leaf ( .. ) => 1,
            CritBitNode::Internal( ( ref left, ref right ), _ ) => {
                left.iter().chain(right.iter()).map(|x| x.len()).fold(0, Add::add)
            }
        }
    }

    fn get( &self, key: &K ) -> Option<&V> {
        match *self {
            CritBitNode::Leaf ( ref k, ref v ) if *k == *key =>
                Some( v ),
            CritBitNode::Internal ( ( Some( ref left ), _ ), ref crit ) if ! bit_at( key, crit ) =>
                left.get( key ),
            CritBitNode::Internal ( ( _, Some( ref right ) ), ref crit ) if   bit_at( key, crit ) =>
                right.get( key ),
            _ => None
        }
    }

    fn get_mut( &mut self, key: &K ) -> Option<&mut V> {
        match *self {
            CritBitNode::Leaf ( ref k, ref mut v ) if *k == *key =>
                Some( v ),
            CritBitNode::Internal ( ( Some( ref mut kid ), _ ), ref crit ) if ! bit_at( key, crit ) =>
                kid.get_mut( key ),
            CritBitNode::Internal ( ( _, Some( ref mut kid ) ), ref crit ) if   bit_at( key, crit ) =>
                kid.get_mut( key ),
            _ => None
        }
    }

    fn insert( &mut self, key: K, value: V ) -> Option<V> {
        match *self {
            CritBitNode::Leaf ( ref k, ref mut v ) if *k == key => {
                Some( std::mem::replace( v, value ) )
            }
            CritBitNode::Leaf ( .. ) => {
                if let CritBitNode::Leaf ( k, v ) = std::mem::replace( self, CritBitNode::Internal( ( None, None ), 0 ) ) {
                    let crit = (k ^ key).leading_zeros();
                    std::mem::replace(self, CritBitNode::Internal (
                        if k < key {
                            (
                                Some( Box::new( CritBitNode::Leaf ( k, v ) ) ),
                                Some( Box::new( CritBitNode::Leaf ( key, value ) ) ),
                            )
                        } else {
                            (
                                Some( Box::new( CritBitNode::Leaf ( key, value ) ) ),
                                Some( Box::new( CritBitNode::Leaf ( k, v ) ) ),
                            )
                        }, crit
                    ));
                } else {
                    unreachable!()
                }
                None
            },
            CritBitNode::Internal ( ( Some( ref mut kid ), _ ), ref crit ) if ! bit_at( &key, crit ) =>
                kid.insert( key, value ),
            CritBitNode::Internal ( ( _, Some( ref mut kid ) ), ref crit ) if   bit_at( &key, crit ) =>
                kid.insert( key, value ),
            _ => unreachable!()
        }
    }
}

#[test]
fn verify_bit_at() {
    assert_eq!( bit_at( &1u8, &0u32 ), false );
    assert_eq!( bit_at( &128u8, &0u32 ), true );
    assert_eq!( bit_at( &1u8, &7u32 ), true );
    assert_eq!( bit_at( &128u8, &7u32 ), false );
}

#[test]
fn empty_len() {
    let t : CritBit<u8,()> = CritBit::new();
    assert_eq!( t.len(), 0 );
}

#[test]
fn empty_contains_key() {
    let t : CritBit<u8,()> = CritBit::new();
    assert_eq!( t.contains_key( &0u8 ), false );
    assert_eq!( t.contains_key( &128u8 ), false );
    assert_eq!( t.contains_key( &255u8 ), false );
}

#[test]
fn empty_get() {
    let t : CritBit<u8,()> = CritBit::new();
    assert_eq!( t.get( &0u8 ), None );
    assert_eq!( t.get( &128u8 ), None );
    assert_eq!( t.get( &255u8 ), None );
}

#[test]
fn empty_get_mut() {
    let mut t : CritBit<u8,()> = CritBit::new();
    assert!( t.get_mut( &0u8 ).is_none() );
    assert!( t.get_mut( &128u8 ).is_none() );
    assert!( t.get_mut( &255u8 ).is_none() );
}

#[test]
fn insert_len() {
    let mut t : CritBit<u8,()> = CritBit::new();
    assert_eq!( t.len(), 0 );

    t.insert( 0u8, () );
    assert_eq!( t.len(), 1 )
}

#[test]
fn insert_contains_key() {
    let mut t : CritBit<u8,()> = CritBit::new();
    assert_eq!( t.contains_key( &0u8 ), false );

    t.insert( 0u8, () );
    assert_eq!( t.contains_key( &0u8 ), true );
}

#[test]
fn insert_get() {
    let mut t : CritBit<u8,u8> = CritBit::new();
    assert_eq!( t.get( &0u8 ), None );

    t.insert( 0u8, 1u8 );
    assert_eq!( t.get( &0u8 ), Some ( &1u8 ) );
}

#[test]
fn insert_get_mut() {
    let mut t : CritBit<u8,u8> = CritBit::new();
    assert_eq!( t.get( &0u8 ), None );

    t.insert( 0u8, 1u8 );
    assert_eq!( t.get( &0u8 ), Some ( &1u8 ) );

    *t.get_mut( &0u8 ).unwrap() = 2u8;
    assert_eq!( t.get( &0u8 ), Some ( &2u8 ) );
}

#[test]
fn insert_insert() {
    let mut t : CritBit<u8,u8> = CritBit::new();
    assert_eq!( t.get( &0u8 ), None );

    t.insert( 0u8, 1u8 );
    assert_eq!( t.get( &0u8 ), Some ( &1u8 ) );

    assert_eq!( t.insert( 0u8, 2u8 ), Some ( 1u8 ) );
    assert_eq!( t.get( &0u8 ), Some ( &2u8 ) );
}
