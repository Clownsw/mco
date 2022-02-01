/// macro used to spawn a coroutine
///
/// this macro is just a convenient wrapper for [`spawn`].
/// However the supplied coroutine block is not wrapped in `unsafe` block
///
/// [`spawn`]: coroutine/fn.spawn.html
#[macro_export]
macro_rules! go {
    // for free spawn
    ($func:expr) => {{
        unsafe { $crate::coroutine::spawn($func) }
    }};

    // for builder/scope spawn
    ($builder:expr, $func:expr) => {{
        use $crate::coroutine::Spawn;
        unsafe { $builder.spawn($func) }
    }};

    // for cqueue add spawn
    ($cqueue:expr, $token:expr, $func:expr) => {{
        unsafe { $cqueue.add($token, $func) }
    }};
}

/// macro used to spawn a coroutine with options such as name, stack_size.
///
/// this macro is just a convenient wrapper for [`spawn`].
/// However the supplied coroutine block is not wrapped in `unsafe` block
///
/// [`spawn`]: coroutine/fn.spawn.html
#[macro_export]
macro_rules! go_with {
    // for stack_size
    ($stack_size:expr, $func:expr) => {{
        fn _go_check<F, T>(stack_size: usize, f: F) -> F
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
        {
            f
        }
        let f = _go_check($stack_size, $func);
        let builder = $crate::coroutine::Builder::new().stack_size($stack_size);
        unsafe { builder.spawn(f) }
    }};

    // for name and stack_size
    ($name: expr, $stack_size:expr, $func:expr) => {{
        fn _go_check<F, T>(name: &str, stack_size: usize, f: F) -> F
        where
            F: FnOnce() -> T + Send + 'static,
            T: Send + 'static,
        {
            f
        }
        let f = _go_check($name, $stack_size, $func);
        let builder = $crate::coroutine::Builder::new()
            .name($name.to_owned())
            .stack_size($stack_size);
        unsafe { builder.spawn(f) }
    }};
}

/// macro used to create the select coroutine
/// that will run in a infinite loop, and generate
/// as many events as possible
#[macro_export]
macro_rules! cqueue_add {
    ($cqueue:ident, $token:expr, $name:pat = $top:expr => $bottom:expr) => {{
        $crate::go!($cqueue, $token, |es| loop {
            let $name = $top;
            es.send(es.get_token());
            $bottom
        })
    }};
}

/// macro used to create the select coroutine
/// that will run only once, thus generate only one event
/// use cogo::select;
///
#[macro_export]
macro_rules! cqueue_add_oneshot {
    ($cqueue:ident, $token:expr, $name:pat = $top:expr => $bottom:expr) => {{
        $crate::go!($cqueue, $token, |es| {
            if let $name = $top{
                $bottom
            }
            es.send(es.get_token());
        })
    }};
}

/// macro used to select for only one event
/// it will return the index of which event happens first
/// for example:
/// ```rust
/// use cogo::{chan, select};
///
///     let (s, r) = chan!();
///     s.send(1);
///     select! {
///         rv = r.recv() => {
///             println!("{:?}",rv);
///         }
///     };
/// ```
#[macro_export]
macro_rules! select {
    (
        $($name:pat = $top:expr => $bottom:expr), +$(,)?
    ) => ({
        use $crate::cqueue;
        cqueue::scope(|cqueue| {
            let mut _token = 0;
            $(
                $crate::cqueue_add_oneshot!(cqueue, _token, $name = $top => $bottom);
                _token += 1;
            )+
            match cqueue.poll(None) {
                Ok(ev) => return ev.token,
                _ => unreachable!("select error"),
            }
        })
    })
}

/// macro used to join all scoped sub coroutines
/// for example:
/// ```rust
/// use cogo::join;
/// join!({  },
///       {  },
///       {  }
/// );
/// ```
#[macro_export]
macro_rules! join {
    (
        $($body:expr),+
    ) => ({
        use $crate::coroutine;
        coroutine::scope(|s| {
            $(
                $crate::go!(s, || $body);
            )+
        })
    })
}

/// A macro to create a `static` of type `LocalKey`
///
/// This macro is intentionally similar to the `thread_local!`, and creates a
/// `static` which has a `with` method to access the data on a coroutine.
///
/// The data associated with each coroutine local is per-coroutine,
/// so different coroutines will contain different values.
#[macro_export]
macro_rules! coroutine_local {
    (static $NAME:ident : $t:ty = $e:expr) => {
        static $NAME: $crate::LocalKey<$t> = {
            fn __init() -> $t {
                $e
            }
            fn __key() -> ::std::any::TypeId {
                struct __A;
                ::std::any::TypeId::of::<__A>()
            }
            $crate::LocalKey {
                __init: __init,
                __key: __key,
            }
        };
    };
}
