use std::num::Bitwise;

pub enum CritBit<K,V> {
    Leaf ( K, V ),
    Internal ( (Box<CritBit<K,V>>, Box<CritBit<K,V>>), K ),
    Empty
}

#[inline(always)]
fn bit_at<T: Bitwise + Eq>( value: &T, pos: &T ) -> bool {
    (*value << *pos).leading_zeros() == ( *value & !*value )
}

impl<K: Bitwise + Eq, V> Container for CritBit<K, V> {
    fn len( &self ) -> uint {
        match *self {
            Empty => 0,
            Leaf ( .. ) => 1,
            Internal( ( ref left, ref right ), _ ) => {
                left.len() + right.len()
            }
        }
    }
}

impl<K: Bitwise + Eq, V> Map<K,V> for CritBit<K,V> {
    fn find<'a>( &'a self, key: &K ) -> Option<&'a V> {
        match *self {
            Leaf ( ref k, ref v ) if *k == *key =>
                Some( v ),
            Internal ( ( ref left, _ ), ref crit ) if ! bit_at( key, crit ) =>
                left.find( key ),
            Internal ( ( _, ref right ), ref crit ) if   bit_at( key, crit ) =>
                right.find( key ),
            _ => None
        }
    }

    fn contains_key( &self, key: &K ) -> bool {
        match self.find( key ) {
            Some( _ ) => true,
            None      => false
        }
    }
}

impl<K: Bitwise + Eq, V> Mutable for CritBit<K,V> {
    fn clear( &mut self ) {
        *self = Empty
    }
}

impl<K: Bitwise + Eq, V> MutableMap<K,V> for CritBit<K,V> {
    fn find_mut<'a>( &'a mut self, key: &K ) -> Option<&'a mut V> {
        match *self {
            Leaf ( ref k, ref mut v ) if *k == *key =>
                Some( v ),
            Internal ( ref mut children, ref crit ) if ! bit_at( key, crit ) =>
                children.0.find_mut( key ),
            Internal ( ref mut children, ref crit ) if   bit_at( key, crit ) =>
                children.1.find_mut( key ),
            _ => None
        }
    }

    fn pop( &mut self, key: &K ) -> Option<V> {
        let mut val = std::mem::replace( self, Empty );
        let ret = match val {
            Internal ( ref mut children, ref crit ) if ! bit_at( key, crit ) =>
                children.0.pop( key ),
            Internal ( ref mut children, ref crit ) if   bit_at( key, crit ) =>
                children.1.pop( key ),
            _ => None
        };

        match val {
            Leaf ( k, v ) => {
                if k == *key {
                    Some ( v )
                } else {
                    std::mem::replace( self, Leaf ( k, v ) );
                    None
                }
            }
            Internal ( ( &Empty, kid ), _ ) => {
                std::mem::replace( self, kid );
                ret
            },
            Internal ( ( kid, &Empty ), _ ) => {
                std::mem::replace( self, kid );
                ret
            },
            _ => {
                std::mem::replace( self, val );
                ret
            }
        }
    }

    fn swap( &mut self, key: K, value: V ) -> Option<V> {
        let val = std::mem::replace( self, Empty );
        match val {
            Leaf ( k, v ) => {
                let crit = ( k ^ key ).leading_zeros();
                let bit = bit_at( &key, &crit );
                if k == key {
                    std::mem::replace( self, Leaf ( key, value ) );
                    Some( v )
                } else if bit {
                    std::mem::replace( self, Internal (
                        ( Box::new( Leaf ( k, v ) ), Box::new( Leaf ( key, value ) ) ), crit
                    ) );
                    None
                } else {
                    std::mem::replace( self, Internal (
                        ( Box::new( Leaf ( key, value ) ), Box::new( Leaf ( k, v ) ) ), crit
                    ) );
                    None
                }
            },
            Internal ( .. ) => {
                std::mem::replace( self, val );
                match *self {
                    Internal ( ref mut children, ref crit ) if ! bit_at( &key, crit ) =>
                        children.0.swap( key, value ),
                    Internal ( ref mut children, ref crit ) if   bit_at( &key, crit ) =>
                        children.1.swap( key, value ),
                    _ => None
                }

            },
            Empty => {
                std::mem::replace( self, Leaf ( key, value ) );
                None
            }
        }
    }
}

#[test]
fn verify_bit_at() {
    assert_eq!( bit_at( &1u8, &0u8 ), false );
    assert_eq!( bit_at( &128u8, &0u8 ), true );
    assert_eq!( bit_at( &1u8, &7u8 ), true );
    assert_eq!( bit_at( &128u8, &7u8 ), false );
}

#[test]
fn empty_len() {
    let t : CritBit<u8,()> = Empty;
    assert_eq!( t.len(), 0 );
}

#[test]
fn empty_contains_key() {
    let t : CritBit<u8,()> = Empty;
    assert_eq!( t.contains_key( &0u8 ), false );
    assert_eq!( t.contains_key( &128u8 ), false );
    assert_eq!( t.contains_key( &255u8 ), false );
}

#[test]
fn empty_find() {
    let t : CritBit<u8,()> = Empty;
    assert_eq!( t.find( &0u8 ), None );
    assert_eq!( t.find( &128u8 ), None );
    assert_eq!( t.find( &255u8 ), None );
}

#[test]
fn leaf_len() {
    let t : CritBit<u8,()> = Leaf ( 0u8, () );
    assert_eq!( t.len(), 1 )
}

#[test]
fn leaf_contains_key() {
    let t : CritBit<u8,()> = Leaf ( 0u8, () );
    assert_eq!( t.contains_key( &0u8 ), true );
    assert_eq!( t.contains_key( &128u8 ), false );
    assert_eq!( t.contains_key( &255u8 ), false );
}

#[test]
fn leaf_find() {
    let t : CritBit<u8,u8> = Leaf ( 0u8, 1u8 );
    let val = 1u8;
    assert_eq!( t.find( &0u8 ), Some ( &val ) );
    assert_eq!( t.find( &128u8 ), None );
    assert_eq!( t.find( &255u8 ), None );
}

#[test]
fn internal_len() {
    let t : CritBit<u8,()> = Internal (
        ( Box::new( Leaf ( 0u8, () ) ), Box::new( Leaf ( 128u8, () ) ) ), 0u8
    );
    assert_eq!( t.len(), 2 );
}

#[test]
fn internal_contains_key() {
    let t : CritBit<u8,()> = Internal (
        ( Box::new( Leaf ( 0u8, () ) ), Box::new( Leaf ( 128u8, () ) ) ), 0u8
    );
    assert_eq!( t.contains_key( &0u8 ), true );
    assert_eq!( t.contains_key( &128u8 ), true );
    assert_eq!( t.contains_key( &255u8 ), false );
}

#[test]
fn internal_find() {
    let t : CritBit<u8,u8> = Internal (
        ( Box::new( Leaf ( 0u8, 1u8 ) ), Box::new( Leaf ( 128u8, 1u8 ) ) ), 0u8
    );
    let val = 1u8;
    assert_eq!( t.find( &0u8 ), Some ( &val ) );
    assert_eq!( t.find( &128u8 ), Some ( &val ) );
    assert_eq!( t.find( &255u8 ), None );
}

#[test]
fn leaf_clear() {
    let mut t : CritBit<u8,()> = Leaf ( 0u8, () );
    assert_eq!( t.len(), 1 );
    t.clear();
    assert_eq!( t.len(), 0 );
}

#[test]
fn internal_clear() {
    let mut t : CritBit<u8,()> = Internal (
        ( Box::new( Leaf ( 0u8, () ) ), Box::new( Leaf ( 128u8, () ) ) ), 0u8
    );
    assert_eq!( t.len(), 2 );
    t.clear();
    assert_eq!( t.len(), 0 );
}

#[test]
fn empty_find_mut() {
    let mut t : CritBit<u8,()> = Empty;
    assert!( t.find_mut( &0u8 ).is_none() );
    assert!( t.find_mut( &128u8 ).is_none() );
    assert!( t.find_mut( &255u8 ).is_none() );
}

#[test]
fn leaf_find_mut() {
    let mut t : CritBit<u8,u8> = Leaf ( 0u8, 1u8 );
    let val = 7u8;
    {
        let x = t.find_mut( &0u8 );
        assert!( x.is_some() );
        let y = x.unwrap();
        assert_eq!( *y, 1u8 );
        *y = 7u8;
    }

    assert_eq!( t.find( &0u8 ), Some( &val ) );
    assert!( t.find_mut( &128u8 ).is_none() );
    assert!( t.find_mut( &255u8 ).is_none() );
}

#[test]
fn internal_find_mut() {
    let mut t : CritBit<u8,u8> = Internal (
        ( Box::new( Leaf ( 0u8, 1u8 ) ), Box::new( Leaf ( 128u8, 1u8 ) ) ), 0u8
    );
    let val = 7u8;
    {
        let x = t.find_mut( &0u8 );
        assert!( x.is_some() );
        let y = x.unwrap();
        assert_eq!( *y, 1u8 );
        *y = 7u8;
    }

    assert_eq!( t.find( &0u8 ), Some ( &val ) );
    assert!( t.find_mut( &128u8 ).is_some() );
    assert!( t.find_mut( &255u8 ).is_none() );
}

#[test]
fn empty_swap() {
    let mut t : CritBit<u8,u8> = Empty;
    let val = 1u8;
    assert_eq!( t.swap( 0u8, 1u8 ), None );
    assert_eq!( t.find( &0u8 ), Some ( &val ) );
}

#[test]
fn leaf_swap_exists() {
    let mut t : CritBit<u8,u8> = Leaf ( 0u8, 1u8 );
    let val = 7u8;
    assert_eq!( t.swap( 0u8, 7u8 ), Some ( 1u8 ) );
    assert_eq!( t.find( &0u8 ), Some ( &val ) );
}

#[test]
fn leaf_swap_new() {
    let mut t : CritBit<u8,u8> = Leaf ( 0u8, 1u8 );
    let oldval = 1u8;
    let val = 7u8;
    assert_eq!( t.swap( 128u8, 7u8 ), None );
    assert_eq!( t.len(), 2 );
    assert_eq!( t.find( &0u8 ), Some ( &oldval ) );
    assert_eq!( t.find( &128u8 ), Some ( &val ) );
}

#[test]
fn internal_swap_new() {
    let mut t : CritBit<u8,u8> = Internal (
        ( Box::new( Leaf ( 0u8, 1u8 ) ), Box::new( Leaf ( 128u8, 1u8 ) ) ), 0u8
    );
    let oldval = 1u8;
    let val = 7u8;
    assert_eq!( t.swap( 255u8, 7u8 ), None );
    assert_eq!( t.len(), 3 );
    assert_eq!( t.find( &0u8 ), Some ( &oldval ) );
    assert_eq!( t.find( &128u8 ), Some ( &oldval ) );
    assert_eq!( t.find( &255u8 ), Some ( &val ) );
}

#[test]
fn internal_swap_exists() {
    let mut t : CritBit<u8,u8> = Internal (
        ( Box::new( Leaf ( 0u8, 1u8 ) ), Box::new( Leaf ( 128u8, 1u8 ) ) ), 0u8
    );
    let val = 7u8;
    assert_eq!( t.swap( 0u8, 7u8 ), Some ( 1u8 ) );
    assert_eq!( t.find( &0u8 ), Some ( &val ) );
}

#[test]
fn empty_pop() {
    let mut t : CritBit<u8,()> = Empty;
    assert_eq!( t.pop( &0u8 ), None );
}

#[test]
fn leaf_pop() {
    let mut t : CritBit<u8,()> = Leaf ( 0u8, () );
    assert_eq!( t.pop( &0u8 ), Some ( () ) );
    assert_eq!( t.len(), 0 );
}

#[test]
fn internal_pop() {
    let mut t : CritBit<u8,()> = Internal (
        ( Box::new( Leaf ( 0u8, () ) ), Box::new( Leaf ( 128u8, () ) ) ), 0u8
    );
    assert_eq!( t.pop( &0u8 ), Some ( () ) );
    assert_eq!( t.len(), 1 );
}
