#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![no_std]

pub mod detail;
mod event;
#[cfg(ngx_feature = "http")]
mod http;
mod queue;
mod rbtree;
#[cfg(ngx_feature = "stream")]
mod stream;
mod string;

use core::ptr;

#[doc(hidden)]
mod bindings {
    #![allow(unknown_lints)] // unnecessary_transmutes
    #![allow(missing_docs)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(clippy::all)]
    #![allow(improper_ctypes)]
    #![allow(rustdoc::broken_intra_doc_links)]
    #![allow(unnecessary_transmutes)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
#[doc(no_inline)]
pub use bindings::*;
pub use event::*;
#[cfg(ngx_feature = "http")]
pub use http::*;
pub use queue::*;
pub use rbtree::*;
#[cfg(ngx_feature = "stream")]
pub use stream::*;

/// Default alignment for pool allocations.
pub const NGX_ALIGNMENT: usize = NGX_RS_ALIGNMENT;

// Check if the allocations made with ngx_palloc are properly aligned.
// If the check fails, objects allocated from `ngx_pool` can violate Rust pointer alignment
// requirements.
const _: () = assert!(core::mem::align_of::<ngx_str_t>() <= NGX_ALIGNMENT);

impl ngx_array_t {
    /// Returns the contents of this array as a slice of `T`.
    ///
    /// # Safety
    ///
    /// The array must be a valid, initialized array containing elements of type T or compatible in
    /// layout with T (e.g. `#[repr(transparent)]` wrappers).
    pub unsafe fn as_slice<T>(&self) -> &[T] {
        debug_assert_eq!(
            core::mem::size_of::<T>(),
            self.size,
            "ngx_array_t::as_slice(): element size mismatch"
        );
        if self.nelts == 0 {
            &[]
        } else {
            // SAFETY: in a valid array, `elts` is a valid well-aligned pointer to at least `nelts`
            // elements of size `size`
            core::slice::from_raw_parts(self.elts.cast(), self.nelts)
        }
    }

    /// Returns the contents of this array as a mutable slice of `T`.
    ///
    /// # Safety
    ///
    /// The array must be a valid, initialized array containing elements of type T or compatible in
    /// layout with T (e.g. `#[repr(transparent)]` wrappers).
    pub unsafe fn as_slice_mut<T>(&mut self) -> &mut [T] {
        debug_assert_eq!(
            core::mem::size_of::<T>(),
            self.size,
            "ngx_array_t::as_slice_mut(): element size mismatch"
        );
        if self.nelts == 0 {
            &mut []
        } else {
            // SAFETY: in a valid array, `elts` is a valid well-aligned pointer to at least `nelts`
            // elements of size `size`
            core::slice::from_raw_parts_mut(self.elts.cast(), self.nelts)
        }
    }
}

impl ngx_command_t {
    /// Creates a new empty [`ngx_command_t`] instance.
    ///
    /// This method replaces the `ngx_null_command` C macro. This is typically used to terminate an
    /// array of configuration directives.
    ///
    /// [`ngx_command_t`]: https://nginx.org/en/docs/dev/development_guide.html#config_directives
    pub const fn empty() -> Self {
        Self {
            name: ngx_str_t::empty(),
            type_: 0,
            set: None,
            conf: 0,
            offset: 0,
            post: ptr::null_mut(),
        }
    }
}

impl ngx_module_t {
    /// Create a new `ngx_module_t` instance with default values.
    pub const fn default() -> Self {
        Self {
            ctx_index: ngx_uint_t::MAX,
            index: ngx_uint_t::MAX,
            name: ptr::null_mut(),
            spare0: 0,
            spare1: 0,
            version: nginx_version as ngx_uint_t,
            signature: NGX_RS_MODULE_SIGNATURE.as_ptr(),
            ctx: ptr::null_mut(),
            commands: ptr::null_mut(),
            type_: 0,
            init_master: None,
            init_module: None,
            init_process: None,
            init_thread: None,
            exit_thread: None,
            exit_process: None,
            exit_master: None,
            spare_hook0: 0,
            spare_hook1: 0,
            spare_hook2: 0,
            spare_hook3: 0,
            spare_hook4: 0,
            spare_hook5: 0,
            spare_hook6: 0,
            spare_hook7: 0,
        }
    }
}

impl ngx_variable_value_t {
    /// Returns the contents of this variable value as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        match self.len() {
            0 => &[],
            // SAFETY: data for non-empty value must be a valid well-aligned pointer.
            len => unsafe { core::slice::from_raw_parts(self.data, len as usize) },
        }
    }
}

impl AsRef<[u8]> for ngx_variable_value_t {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Returns the error code of the last failed operation (`errno`).
#[inline]
pub fn ngx_errno() -> ngx_err_t {
    // SAFETY: GetLastError takes no arguments and reads a thread-local variable
    #[cfg(windows)]
    let err = unsafe { GetLastError() };

    #[cfg(not(windows))]
    let err = errno::errno().0;

    err as ngx_err_t
}

/// Sets the error code (`errno`).
#[inline]
pub fn ngx_set_errno(err: ngx_err_t) {
    #[cfg(windows)]
    // SAFETY: SetLastError takes one argument by value and updates a thread-local variable
    unsafe {
        SetLastError(err as _)
    }
    #[cfg(not(windows))]
    errno::set_errno(errno::Errno(err as _))
}

/// Returns the error code of the last failed sockets operation.
#[inline]
pub fn ngx_socket_errno() -> ngx_err_t {
    // SAFETY: WSAGetLastError takes no arguments and reads a thread-local variable
    #[cfg(windows)]
    let err = unsafe { WSAGetLastError() };

    #[cfg(not(windows))]
    let err = errno::errno().0;

    err as ngx_err_t
}

/// Sets the error code of the sockets operation.
#[inline]
pub fn ngx_set_socket_errno(err: ngx_err_t) {
    #[cfg(windows)]
    // SAFETY: WSaSetLastError takes one argument by value and updates a thread-local variable
    unsafe {
        WSASetLastError(err as _)
    }
    #[cfg(not(windows))]
    errno::set_errno(errno::Errno(err as _))
}

/// Returns a non cryptograhpically-secure pseudo-random integer.
#[inline]
pub fn ngx_random() -> core::ffi::c_long {
    #[cfg(windows)]
    unsafe {
        // Emulate random() as Microsoft CRT does not provide it.
        // rand() should be thread-safe in the multi-threaded CRT we link to, but will not be seeded
        // outside of the main thread.
        let x: u32 = ((rand() as u32) << 16) ^ ((rand() as u32) << 8) ^ (rand() as u32);
        (0x7fffffff & x) as _
    }
    #[cfg(not(windows))]
    unsafe {
        random()
    }
}

/// Causes the calling thread to relinquish the CPU.
#[inline]
pub fn ngx_sched_yield() {
    #[cfg(windows)]
    unsafe {
        SwitchToThread()
    };
    #[cfg(all(not(windows), ngx_feature = "have_sched_yield"))]
    unsafe {
        sched_yield()
    };
    #[cfg(not(any(windows, ngx_feature = "have_sched_yield")))]
    unsafe {
        usleep(1)
    }
}

/// Returns cached timestamp in seconds, updated at the start of the event loop iteration.
///
/// Can be stale when accessing from threads, see [ngx_time_update].
#[inline]
pub fn ngx_time() -> time_t {
    // SAFETY: ngx_cached_time is initialized before any module code can run
    unsafe { (*ngx_cached_time).sec }
}

/// Returns cached time, updated at the start of the event loop iteration.
///
/// Can be stale when accessing from threads, see [ngx_time_update].
/// A cached reference to the ngx_timeofday() result is guaranteed to remain unmodified for the next
/// NGX_TIME_SLOTS seconds.
#[inline]
pub fn ngx_timeofday() -> &'static ngx_time_t {
    // SAFETY: ngx_cached_time is initialized before any module code can run
    unsafe { &*ngx_cached_time }
}

/// Initialize a list, using a pool for the backing memory, with capacity to store the given number
/// of elements and element size.
///
/// # Safety
/// * `list` must be non-null
/// * `pool` must be a valid pool
#[inline]
pub unsafe fn ngx_list_init(
    list: *mut ngx_list_t,
    pool: *mut ngx_pool_t,
    n: ngx_uint_t,
    size: usize,
) -> ngx_int_t {
    unsafe {
        (*list).part.elts = ngx_palloc(pool, n * size);
        if (*list).part.elts.is_null() {
            return NGX_ERROR as ngx_int_t;
        }
        (*list).part.nelts = 0;
        (*list).part.next = ptr::null_mut();
        (*list).last = ptr::addr_of_mut!((*list).part);
        (*list).size = size;
        (*list).nalloc = n;
        (*list).pool = pool;
        NGX_OK as ngx_int_t
    }
}

/// Add a key-value pair to an nginx table entry (`ngx_table_elt_t`) in the given nginx memory pool.
///
/// # Arguments
///
/// * `table` - A pointer to the nginx table entry (`ngx_table_elt_t`) to modify.
/// * `pool` - A pointer to the nginx memory pool (`ngx_pool_t`) for memory allocation.
/// * `key` - The key string to add to the table entry.
/// * `value` - The value string to add to the table entry.
///
/// # Safety
/// This function is marked as unsafe because it involves raw pointer manipulation and direct memory
/// allocation using `str_to_uchar`.
///
/// # Returns
/// An `Option<()>` representing the result of the operation. `Some(())` indicates success, while
/// `None` indicates a null table pointer.
///
/// # Example
/// ```rust
/// # use nginx_sys::*;
/// # unsafe fn example(pool: *mut ngx_pool_t, headers: *mut ngx_list_t) {
/// // Obtain a pointer to the nginx table entry
/// let table: *mut ngx_table_elt_t = ngx_list_push(headers).cast();
/// assert!(!table.is_null());
/// let key: &str = "key"; // The key to add
/// let value: &str = "value"; // The value to add
/// let result = add_to_ngx_table(table, pool, key, value);
/// # }
/// ```
pub unsafe fn add_to_ngx_table(
    table: *mut ngx_table_elt_t,
    pool: *mut ngx_pool_t,
    key: impl AsRef<[u8]>,
    value: impl AsRef<[u8]>,
) -> Option<()> {
    if let Some(table) = table.as_mut() {
        let key = key.as_ref();
        table.key = ngx_str_t::from_bytes(pool, key)?;
        table.value = ngx_str_t::from_bytes(pool, value.as_ref())?;
        table.lowcase_key = ngx_pnalloc(pool, table.key.len).cast();
        if table.lowcase_key.is_null() {
            return None;
        }
        table.hash = ngx_hash_strlow(table.lowcase_key, table.key.data, table.key.len);
        return Some(());
    }
    None
}
