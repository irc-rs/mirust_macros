extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn mirust_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let output_name = input.sig.ident.clone();

    let inputs = input.sig.inputs.clone();
    let block = input.block.clone();

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
            let loadinfo = mirust_sdk::get_loadinfo();

            let (data, parms) = if (loadinfo.m_unicode.into()) {
                let data_str = mirust_sdk::pwstr_to_string(data_ptr as *const u16, loadinfo.m_bytes as usize);
                let parms_str = mirust_sdk::pwstr_to_string(parms_ptr as *const u16, loadinfo.m_bytes as usize);
                (data_str, parms_str)
            } else {
                let data_str = mirust_sdk::pstr_to_string(data_ptr as *const u8, loadinfo.m_bytes as usize);
                let parms_str = mirust_sdk::pstr_to_string(parms_ptr as *const u8, loadinfo.m_bytes as usize);
                (data_str, parms_str)
            };

            let closure = |#inputs| #block;

            let result = closure(m_wnd, a_wnd, data, parms, show, nopause);

            if loadinfo.m_unicode.into() {
                if let Some(ref data_str) = result.data {
                    mirust_sdk::string_to_pwstr(data_str, data_ptr as *const u16, loadinfo.m_bytes as usize);
                }
                if let Some(ref parms_str) = result.parms {
                    mirust_sdk::string_to_pwstr(parms_str, parms_ptr as *const u16, loadinfo.m_bytes as usize);
                }
            } else {
                if let Some(ref data_str) = result.data {
                    mirust_sdk::string_to_pstr(data_str, data_ptr as *const u8, loadinfo.m_bytes as usize);
                }
                if let Some(ref parms_str) = result.parms {
                    mirust_sdk::string_to_pstr(parms_str, parms_ptr as *const u8, loadinfo.m_bytes as usize);
                }
            }

            result.code
        }
    };

    output.into()
}