/*
 * SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
 *
 * SPDX-License-Identifier: 0BSD OR CC0-1.0
 */

/*
 * Imports that ReactOS lacks, linked into the program in place of the
 * import library entries.  Each function first looks for the real one at
 * run time, in its own DLL and then in that DLL's private *_vista.dll
 * companion on ReactOS, so that the program still uses the system's
 * version where there is one, on Windows too.  Without one it falls back to
 * an older equivalent, or fails the way a caller already expects.
 */

#define _WIN32_WINNT 0x0A00
#define INITGUID
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winternl.h>
#include <shlobj.h>
#include <knownfolders.h>
#include <math.h>
#include <string.h>
#include <wctype.h>

#ifdef _WIN64
#define IMP(name, bytes) __asm__("__imp_" #name)
#define SYM(name, bytes) __asm__(#name)
#else
#define IMP(name, bytes) __asm__("__imp__" #name "@" #bytes)
#define SYM(name, bytes) __asm__("_" #name "@" #bytes)
#endif

#define MISSING ((FARPROC)(INT_PTR)-1)

/* DLLs of ReactOS with a TLS directory, looked up only if the program
 * already loaded them.  ReactOS gives a DLL that loads after startup no
 * TLS slot, and once advapi32.dll and its dependencies loaded that way,
 * threads started afterwards failed at exit in Rust's TLS destructors. */
static const char *const startup_only[] = { "combase.dll", "ws2_32.dll" };

static FARPROC find(const char *dll, const char *name)
{
    HMODULE module = GetModuleHandleA(dll);
    BOOL loadable = TRUE;
    for (size_t i = 0; i < sizeof startup_only / sizeof *startup_only; i++)
        loadable = loadable && lstrcmpiA(dll, startup_only[i]) != 0;
    if (!module && loadable)
        module = LoadLibraryA(dll);
    return module ? GetProcAddress(module, name) : NULL;
}

/* Function name of dll, or else of its companion, which for API sets is
 * kernel32_vista.dll.  Only the companion of the function's own DLL loads,
 * since user32_vista.dll would bring user32.dll into console programs. */
static FARPROC resolve(const char *dll, const char *name)
{
    char companion[64];
    const char *dot = strchr(dll, '.');
#ifdef ROSSHIM_FALLBACK /* Fallbacks alone, to try them on Windows or Wine */
    (void)dll, (void)name, (void)companion, (void)dot;
    return MISSING;
#endif
    FARPROC proc = find(dll, name);
    if (proc)
        return proc;
    if (strncmp(dll, "api-ms-", 7) == 0)
        proc = find("kernel32_vista.dll", name);
    else if (dot && dot - dll < 32) {
        memcpy(companion, dll, dot - dll);
        strcpy(companion + (dot - dll), "_vista.dll");
        proc = find(companion, name);
    }
    return proc ? proc : MISSING;
}

/* Function name of dll with its parameters and their names, which fall back
 * to the statements of the last argument */
#define SHIM(ret, name, bytes, dll, params, args, fallback)                   \
    ret WINAPI shim_##name params SYM(name, bytes);                           \
    ret WINAPI shim_##name params                                             \
    {                                                                         \
        static FARPROC real;                                                  \
        if (!real)                                                            \
            real = resolve(dll, #name);                                       \
        if (real != MISSING)                                                  \
            return ((ret (WINAPI *) params)(void (*)(void))real) args;        \
        fallback                                                              \
    }                                                                         \
    void *imp_##name IMP(name, bytes) = (void *)shim_##name;

#define FAIL(value)                                                           \
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);                                 \
    return value;

/* Synchronization of Rust's standard library */

typedef NTSTATUS (NTAPI *wait_fn)(const volatile void *, const void *, SIZE_T,
                                  const LARGE_INTEGER *);
typedef void (NTAPI *wake_fn)(const void *);

static BOOL wait_on_address(volatile void *address, void *compare, SIZE_T size,
                            DWORD ms)
{
    static FARPROC rtl;
    if (!rtl)
        rtl = resolve("ntdll.dll", "RtlWaitOnAddress");
    if (rtl != MISSING) {
        LARGE_INTEGER timeout = { .QuadPart = -(LONGLONG)ms * 10000 };
        NTSTATUS status = ((wait_fn)(void (*)(void))rtl)(
            address, compare, size, ms == INFINITE ? NULL : &timeout);
        if (status == STATUS_TIMEOUT) {
            SetLastError(ERROR_TIMEOUT);
            return FALSE;
        }
        return TRUE;
    }
    /* Polling, which callers take as a spurious wake */
    if (memcmp((const void *)address, compare, size) != 0)
        return TRUE;
    if (ms == 0) {
        SetLastError(ERROR_TIMEOUT);
        return FALSE;
    }
    Sleep(1);
    return TRUE;
}

static void wake(FARPROC *rtl, const char *name, void *address)
{
    if (!*rtl)
        *rtl = resolve("ntdll.dll", name);
    if (*rtl != MISSING)
        ((wake_fn)(void (*)(void))*rtl)(address);
}

static FARPROC wake_single, wake_all;

SHIM(BOOL, WaitOnAddress, 16, "api-ms-win-core-synch-l1-2-0.dll",
     (volatile void *address, void *compare, SIZE_T size, DWORD ms),
     (address, compare, size, ms),
     return wait_on_address(address, compare, size, ms);)
SHIM(void, WakeByAddressSingle, 4, "api-ms-win-core-synch-l1-2-0.dll",
     (void *address), (address),
     wake(&wake_single, "RtlWakeAddressSingle", address);)
SHIM(void, WakeByAddressAll, 4, "api-ms-win-core-synch-l1-2-0.dll",
     (void *address), (address),
     wake(&wake_all, "RtlWakeAddressAll", address);)

/* Locks and modules of LLVM's libunwind, which kernel32_vista.dll and
 * psapi.dll of ReactOS hold */

SHIM(void, AcquireSRWLockExclusive, 4, "kernel32.dll", (void *lock), (lock),
     return;)
SHIM(void, AcquireSRWLockShared, 4, "kernel32.dll", (void *lock), (lock),
     return;)
SHIM(void, ReleaseSRWLockExclusive, 4, "kernel32.dll", (void *lock), (lock),
     return;)
SHIM(void, ReleaseSRWLockShared, 4, "kernel32.dll", (void *lock), (lock),
     return;)

static BOOL enum_process_modules(HANDLE process, HMODULE *modules, DWORD size,
                                 DWORD *needed)
{
    FARPROC psapi = find("psapi.dll", "EnumProcessModules");
    if (!psapi) {
        SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        return FALSE;
    }
    return ((BOOL (WINAPI *)(HANDLE, HMODULE *, DWORD, DWORD *))(
        void (*)(void))psapi)(process, modules, size, needed);
}

SHIM(BOOL, K32EnumProcessModules, 16, "kernel32.dll",
     (HANDLE process, HMODULE *modules, DWORD size, DWORD *needed),
     (process, modules, size, needed),
     return enum_process_modules(process, modules, size, needed);)

/*
 * Fiber local storage of Rust's TLS destructors.  ReactOS runs a thread's
 * FLS callbacks only after it has freed the thread's TLS blocks, while
 * Rust's callback drains a destructor list kept in TLS, which then reads
 * and writes freed memory and corrupts the heap.  Where ntdll shows the FLS
 * of Windows, the calls go through.  Elsewhere each FLS key is a TLS slot,
 * and the callbacks run from this program's own TLS callback, before the
 * loader frees TLS, in the order that Windows keeps.
 */

#define FLS_KEYS 64

static PFLS_CALLBACK_FUNCTION fls_callbacks[FLS_KEYS];
static DWORD fls_slots[FLS_KEYS];
static volatile LONG fls_ready[FLS_KEYS];
static volatile LONG fls_used;

static BOOL fls_native(void)
{
    static volatile LONG native = -1;
#ifdef ROSSHIM_FALLBACK
    native = FALSE;
#endif
    if (native < 0) {
        HMODULE ntdll = GetModuleHandleA("ntdll.dll");
        native = GetProcAddress(ntdll, "RtlProcessFlsData") ||
                 GetProcAddress(ntdll, "RtlFlsAlloc");
    }
    return native;
}

#define NATIVE(ret, name, params, args)                                       \
    if (fls_native())                                                         \
        return ((ret (WINAPI *) params)(void (*)(void))find("kernel32.dll",   \
                                                            #name)) args;

static BOOL fls_key(DWORD key)
{
    if (key < FLS_KEYS && fls_ready[key])
        return TRUE;
    SetLastError(ERROR_INVALID_PARAMETER);
    return FALSE;
}

/* Callback of key for the current thread, if it holds a value */
static void fls_run(DWORD key)
{
    void *value = TlsGetValue(fls_slots[key]);
    if (value && fls_callbacks[key]) {
        TlsSetValue(fls_slots[key], NULL);
        fls_callbacks[key](value);
    }
}

DWORD WINAPI shim_FlsAlloc(PFLS_CALLBACK_FUNCTION callback) SYM(FlsAlloc, 4);
DWORD WINAPI shim_FlsAlloc(PFLS_CALLBACK_FUNCTION callback)
{
    NATIVE(DWORD, FlsAlloc, (PFLS_CALLBACK_FUNCTION), (callback))
    LONG key = InterlockedIncrement(&fls_used) - 1;
    DWORD slot = key < FLS_KEYS ? TlsAlloc() : TLS_OUT_OF_INDEXES;
    if (slot == TLS_OUT_OF_INDEXES) {
        SetLastError(ERROR_NO_MORE_ITEMS);
        return FLS_OUT_OF_INDEXES;
    }
    fls_slots[key] = slot;
    fls_callbacks[key] = callback;
    InterlockedExchange(&fls_ready[key], 1);
    return key;
}
void *imp_FlsAlloc IMP(FlsAlloc, 4) = (void *)shim_FlsAlloc;

BOOL WINAPI shim_FlsFree(DWORD key) SYM(FlsFree, 4);
BOOL WINAPI shim_FlsFree(DWORD key)
{
    NATIVE(BOOL, FlsFree, (DWORD), (key))
    if (!fls_key(key))
        return FALSE;
    fls_run(key);
    InterlockedExchange(&fls_ready[key], 0);
    return TlsFree(fls_slots[key]);
}
void *imp_FlsFree IMP(FlsFree, 4) = (void *)shim_FlsFree;

BOOL WINAPI shim_FlsSetValue(DWORD key, void *value) SYM(FlsSetValue, 8);
BOOL WINAPI shim_FlsSetValue(DWORD key, void *value)
{
    NATIVE(BOOL, FlsSetValue, (DWORD, void *), (key, value))
    return fls_key(key) && TlsSetValue(fls_slots[key], value);
}
void *imp_FlsSetValue IMP(FlsSetValue, 8) = (void *)shim_FlsSetValue;

void *WINAPI shim_FlsGetValue(DWORD key) SYM(FlsGetValue, 4);
void *WINAPI shim_FlsGetValue(DWORD key)
{
    NATIVE(void *, FlsGetValue, (DWORD), (key))
    return fls_key(key) ? TlsGetValue(fls_slots[key]) : NULL;
}
void *imp_FlsGetValue IMP(FlsGetValue, 4) = (void *)shim_FlsGetValue;

static void NTAPI fls_rundown(void *module, DWORD reason, void *reserved)
{
    (void)module, (void)reserved;
    if (reason != DLL_THREAD_DETACH || fls_native())
        return;
    for (LONG key = 0; key < fls_used && key < FLS_KEYS; key++)
        if (fls_ready[key])
            fls_run(key);
}

__attribute__((section(".CRT$XLY"), used))
const PIMAGE_TLS_CALLBACK rosshim_fls_rundown = fls_rundown;

/*
 * Math of the C runtime that ReactOS's ucrtbase.dll exports as stubs,
 * which return nothing useful, so that the loader accepts the program and
 * the results are wrong.  With its truncf, cosmic-text placed every glyph
 * of iced at the top of the window.  These take log, exp, and sqrt, which
 * ReactOS implements, and replace the stubs on every system.
 */

#ifdef _WIN64
#define CIMP(name) __asm__("__imp_" #name)
#define CSYM(name) __asm__(#name)
#else
#define CIMP(name) __asm__("__imp__" #name)
#define CSYM(name) __asm__("_" #name)
#endif

#define MATH(type, name, value)                                               \
    type shim_##name(type x) CSYM(name);                                      \
    type shim_##name(type x) { return value; }                                \
    void *imp_##name CIMP(name) = (void *)shim_##name;

/* x with the bits of its fraction below the binary point cleared */
static double trunc_double(double x)
{
    union { double f; UINT64 i; } u = { x };
    int shift = (int)(u.i >> 52 & 0x7ff) - 0x3ff + 12;
    if (shift >= 64)
        return x; /* Integral, infinite, or NaN */
    u.i &= ~(~0ULL >> (shift < 12 ? 1 : shift));
    return u.f;
}

static float trunc_float(float x)
{
    union { float f; UINT32 i; } u = { x };
    int shift = (int)(u.i >> 23 & 0xff) - 0x7f + 9;
    if (shift >= 32)
        return x;
    u.i &= ~(~0U >> (shift < 9 ? 1 : shift));
    return u.f;
}

static BOOL negative(double x)
{
    union { double f; UINT64 i; } u = { x };
    return u.i >> 63;
}

/* Kahan's log1p, accurate where 1 + x rounds */
static double log1p_double(double x)
{
    double u = 1.0 + x;
    if (u == 1.0)
        return x;
    if (u <= 0.0 || u - u != 0.0)
        return log(u); /* -inf, NaN, or inf */
    return log(u) * (x / (u - 1.0));
}

static double acosh_double(double x)
{
    if (x > 0x1p28)
        return log(x) + 0.69314718055994530942;
    double t = x - 1.0; /* NaN below 1 through sqrt */
    return log1p_double(t + sqrt(2.0 * t + t * t));
}

static double asinh_double(double x)
{
    double a = negative(x) ? -x : x;
    double y = a > 0x1p28 ? log(a) + 0.69314718055994530942
                          : log1p_double(a + a * a / (1.0 + sqrt(1.0 + a * a)));
    return negative(x) ? -y : y;
}

static double cbrt_double(double x)
{
    if (x == 0.0 || x - x != 0.0)
        return x; /* Zero, infinite, or NaN */
    double a = negative(x) ? -x : x;
    double y = exp(log(a) / 3.0);
    for (int i = 0; i < 2; i++)
        y -= (y * y * y - a) / (3.0 * y * y);
    return negative(x) ? -y : y;
}

MATH(double, trunc, trunc_double(x))
MATH(float, truncf, trunc_float(x))
MATH(double, log1p, log1p_double(x))
MATH(float, log1pf, (float)log1p_double(x))
MATH(double, acosh, acosh_double(x))
MATH(float, acoshf, (float)acosh_double(x))
MATH(double, asinh, asinh_double(x))
MATH(float, asinhf, (float)asinh_double(x))
MATH(double, cbrt, cbrt_double(x))
MATH(float, cbrtf, (float)cbrt_double(x))

/* Randomness of Rust's hash maps */

/* RtlGenRandom, imported so that advapi32.dll loads with the program, for
 * the reason given at startup_only */
BOOLEAN WINAPI SystemFunction036(void *data, ULONG size);

static BOOL gen_random(BYTE *data, SIZE_T size)
{
    for (SIZE_T chunk; size; data += chunk, size -= chunk) {
        chunk = size < 0x10000000 ? size : 0x10000000;
        if (!SystemFunction036(data, (ULONG)chunk))
            return FALSE;
    }
    return TRUE;
}

SHIM(BOOL, ProcessPrng, 8, "bcryptprimitives.dll", (BYTE *data, SIZE_T size),
     (data, size), return gen_random(data, size);)

/* Files, threads, and time */

SHIM(HANDLE, CreateWaitableTimerExW, 16, "kernel32.dll",
     (SECURITY_ATTRIBUTES *attributes, const WCHAR *name, DWORD flags,
      DWORD access),
     (attributes, name, flags, access), FAIL(NULL))
SHIM(BOOL, GetFileInformationByHandleEx, 16, "kernel32.dll",
     (HANDLE file, int class, void *info, DWORD size),
     (file, class, info, size), FAIL(FALSE))
SHIM(BOOL, SetFileInformationByHandle, 16, "kernel32.dll",
     (HANDLE file, int class, void *info, DWORD size),
     (file, class, info, size), FAIL(FALSE))
SHIM(DWORD, GetFinalPathNameByHandleW, 16, "kernel32.dll",
     (HANDLE file, WCHAR *path, DWORD size, DWORD flags),
     (file, path, size, flags), FAIL(0))
SHIM(BOOLEAN, CreateSymbolicLinkW, 12, "kernel32.dll",
     (const WCHAR *link, const WCHAR *target, DWORD flags),
     (link, target, flags), FAIL(FALSE))
SHIM(BOOL, GetUserPreferredUILanguages, 16, "kernel32.dll",
     (DWORD flags, ULONG *count, WCHAR *languages, ULONG *size),
     (flags, count, languages, size), FAIL(FALSE))
SHIM(BOOL, IsThreadAFiber, 0, "kernel32.dll", (void), (), return FALSE;)
SHIM(BOOL, SetThreadErrorMode, 8, "kernel32.dll", (DWORD mode, DWORD *old),
     (mode, old),
     DWORD previous = SetErrorMode(mode);
     if (old)
         *old = previous;
     return TRUE;)
SHIM(void, GetSystemTimePreciseAsFileTime, 4, "kernel32.dll", (FILETIME *time),
     (time), GetSystemTimeAsFileTime(time);)
SHIM(BOOL, InitializeProcThreadAttributeList, 16, "kernel32.dll",
     (void *list, DWORD count, DWORD flags, SIZE_T *size),
     (list, count, flags, size), FAIL(FALSE))
SHIM(BOOL, UpdateProcThreadAttribute, 28, "kernel32.dll",
     (void *list, DWORD flags, DWORD_PTR attribute, void *value, SIZE_T size,
      void *previous, SIZE_T *returned),
     (list, flags, attribute, value, size, previous, returned), FAIL(FALSE))
SHIM(void, DeleteProcThreadAttributeList, 4, "kernel32.dll", (void *list),
     (list), return;)

static int compare_ordinal(const WCHAR *a, int a_size, const WCHAR *b,
                           int b_size, BOOL ignore_case)
{
    size_t a_length = a_size < 0 ? wcslen(a) : (size_t)a_size;
    size_t b_length = b_size < 0 ? wcslen(b) : (size_t)b_size;
    for (size_t i = 0; i < a_length && i < b_length; i++) {
        WCHAR x = ignore_case ? towupper(a[i]) : a[i];
        WCHAR y = ignore_case ? towupper(b[i]) : b[i];
        if (x != y)
            return x < y ? CSTR_LESS_THAN : CSTR_GREATER_THAN;
    }
    return a_length == b_length ? CSTR_EQUAL
           : a_length < b_length ? CSTR_LESS_THAN
                                 : CSTR_GREATER_THAN;
}

SHIM(int, CompareStringOrdinal, 20, "kernel32.dll",
     (const WCHAR *a, int a_size, const WCHAR *b, int b_size, BOOL ignore_case),
     (a, a_size, b, b_size, ignore_case),
     return compare_ordinal(a, a_size, b, b_size, ignore_case);)

SHIM(int, GetHostNameW, 8, "ws2_32.dll", (WCHAR *name, int size), (name, size),
     FAIL(-1))
SHIM(BOOL, RoOriginateErrorW, 12, "combase.dll",
     (HRESULT error, UINT length, const WCHAR *message),
     (error, length, message), return FALSE;)

/* Windows of winit */

SHIM(BOOL, RegisterTouchWindow, 8, "user32.dll", (HWND window, ULONG flags),
     (window, flags), FAIL(FALSE))
SHIM(BOOL, GetTouchInputInfo, 16, "user32.dll",
     (HANDLE input, UINT count, void *inputs, int size),
     (input, count, inputs, size), FAIL(FALSE))
SHIM(BOOL, CloseTouchInputHandle, 4, "user32.dll", (HANDLE input), (input),
     FAIL(FALSE))
SHIM(BOOL, IsProcessDPIAware, 0, "user32.dll", (void), (), return FALSE;)
SHIM(BOOL, SetWindowDisplayAffinity, 8, "user32.dll",
     (HWND window, DWORD affinity), (window, affinity), FAIL(FALSE))
SHIM(HRESULT, DwmEnableBlurBehindWindow, 8, "dwmapi.dll",
     (HWND window, const void *blur), (window, blur), return E_NOTIMPL;)
SHIM(HRESULT, DwmSetWindowAttribute, 16, "dwmapi.dll",
     (HWND window, DWORD attribute, const void *value, DWORD size),
     (window, attribute, value, size), return E_NOTIMPL;)

/* Directories of dirs, dialogs of rfd, and audio of cpal */

static HRESULT known_folder_path(const GUID *id, WCHAR **path)
{
    static const struct {
        const GUID *id;
        int csidl;
    } folders[] = {
        { &FOLDERID_RoamingAppData, CSIDL_APPDATA },
        { &FOLDERID_LocalAppData, CSIDL_LOCAL_APPDATA },
        { &FOLDERID_Profile, CSIDL_PROFILE },
        { &FOLDERID_Documents, CSIDL_PERSONAL },
        { &FOLDERID_Desktop, CSIDL_DESKTOPDIRECTORY },
        { &FOLDERID_Music, CSIDL_MYMUSIC },
        { &FOLDERID_Videos, CSIDL_MYVIDEO },
        { &FOLDERID_Pictures, CSIDL_MYPICTURES },
        { &FOLDERID_Fonts, CSIDL_FONTS },
        { &FOLDERID_ProgramData, CSIDL_COMMON_APPDATA },
    };
    *path = NULL;
    for (size_t i = 0; i < sizeof folders / sizeof *folders; i++) {
        if (!IsEqualGUID(id, folders[i].id))
            continue;
        WCHAR buffer[MAX_PATH];
        HRESULT result = SHGetFolderPathW(NULL, folders[i].csidl, NULL, 0,
                                          buffer);
        if (FAILED(result))
            return result;
        size_t size = (wcslen(buffer) + 1) * sizeof *buffer;
        *path = CoTaskMemAlloc(size);
        if (!*path)
            return E_OUTOFMEMORY;
        memcpy(*path, buffer, size);
        return S_OK;
    }
    return E_INVALIDARG;
}

SHIM(HRESULT, SHGetKnownFolderPath, 16, "shell32.dll",
     (const GUID *id, DWORD flags, HANDLE token, WCHAR **path),
     (id, flags, token, path), return known_folder_path(id, path);)
SHIM(HRESULT, SHCreateItemFromParsingName, 16, "shell32.dll",
     (const WCHAR *path, void *context, const GUID *iid, void **item),
     (path, context, iid, item), *item = NULL; return E_NOTIMPL;)
SHIM(HRESULT, ActivateAudioInterfaceAsync, 20, "mmdevapi.dll",
     (const WCHAR *path, const GUID *iid, void *parameters, void *handler,
      void **operation),
     (path, iid, parameters, handler, operation),
     *operation = NULL; return E_NOTIMPL;)
