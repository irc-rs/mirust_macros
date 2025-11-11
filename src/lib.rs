extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn mirust_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Simple string-based parse of the attribute tokens to extract `dllcall = true/false`.
    // We avoid pulling in proc_macro2 or complex syn parsing here for simplicity.
    let attr_string = attr.to_string();
    let _dllcall = (|| {
        if let Some(pos) = attr_string.find("dllcall") {
            if let Some(eq_pos) = attr_string[pos..].find('=') {
                let rest = &attr_string[pos + eq_pos + 1..];
                let rest = rest.trim_start();
                if rest.starts_with("true") {
                    return true;
                }
                if rest.starts_with("false") {
                    return false;
                }
            }
        }

        false
    })();

    let input = parse_macro_input!(item as ItemFn);
    let output_name = input.sig.ident.clone();

    let inputs = input.sig.inputs.clone();
    let block = input.block.clone();

        // Build a token fragment for the dllcall handling only if the attribute
        // requested it. Example: #[mirust_fn(dllcall = true)]
        let dllcall_body = if _dllcall {
            quote! {
                // Check if we're on mIRC's main thread 
                if mirust::is_main_thread(loadinfo.m_hwnd) {
                    // mIRC called this fn with $dll instead of $dllcall
                    // We shouldn't block the GUI thread, so return 1 to continue.
                    return 1; // Continue
                }
            }
        } else {
            quote! {}
        };

        let output = quote! {
        #[unsafe(no_mangle)]
        pub unsafe extern "system" fn #output_name(
            m_wnd: HWND,
            a_wnd: HWND,
            data_ptr: *mut std::ffi::c_void, // PCWSTR or PWSTR
            parms_ptr: *mut std::ffi::c_void, // PCWSTR or PWSTR
            show: BOOL,
            nopause: BOOL,
        ) -> i32 {
            let loadinfo = mirust::get_loadinfo();

            #dllcall_body

            let (data, parms) = if (loadinfo.m_unicode.into()) {
                let data_str = mirust::pwstr_to_string(data_ptr as *const u16, loadinfo.m_bytes as usize);
                let parms_str = mirust::pwstr_to_string(parms_ptr as *const u16, loadinfo.m_bytes as usize);
                (data_str, parms_str)
            } else {
                let data_str = mirust::pstr_to_string(data_ptr as *const u8, loadinfo.m_bytes as usize);
                let parms_str = mirust::pstr_to_string(parms_ptr as *const u8, loadinfo.m_bytes as usize);
                (data_str, parms_str)
            };

            let closure = |#inputs| #block;

            let result = closure(m_wnd, a_wnd, data, parms, show, nopause);

            if loadinfo.m_unicode.into() {
                if let Some(ref data_str) = result.data {
                    mirust::string_to_pwstr(data_str, data_ptr as *const u16, loadinfo.m_bytes as usize);
                }
                if let Some(ref parms_str) = result.parms {
                    mirust::string_to_pwstr(parms_str, parms_ptr as *const u16, loadinfo.m_bytes as usize);
                }
            } else {
                if let Some(ref data_str) = result.data {
                    mirust::string_to_pstr(data_str, data_ptr as *const u8, loadinfo.m_bytes as usize);
                }
                if let Some(ref parms_str) = result.parms {
                    mirust::string_to_pstr(parms_str, parms_ptr as *const u8, loadinfo.m_bytes as usize);
                }
            }

            result.code
        }
    };

    output.into()
}