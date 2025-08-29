use parking_lot::Mutex;
use reqwest::{StatusCode, blocking::RequestBuilder, header::HeaderMap};
use testangel_engine::{Evidence, EvidenceContent, engine};

use crate::http_evidence::{req_to_evidence, res_to_evidence};

mod http_evidence;

engine! {
    /// Make HTTP requests.
    ///
    /// To make an HTTP request, follow this kind of flow:
    ///
    /// HTTP.PreparePost("url")
    /// HTTP.AddHeader("Authorization", "Bearer <token>")
    /// HTTP.AddBody('{"test": true}')
    /// local bdy = HTTP.Send()
    ///
    /// -- You can now also check the last status code and headers
    /// local sts = HTTP.LastStatus()
    /// local hdr = HTTP.LastRequestHeader("Content-Length")
    #[engine(
        version = env!("CARGO_PKG_VERSION"),
    )]
    #[derive(Default)]
    struct Http {
        /// The reqwest client
        client: reqwest::blocking::Client,

        /// The status code from the last request
        last_status: Option<StatusCode>,
        /// The headers from the last request
        last_headers: Option<HeaderMap>,

        /// The builder for the next request, if one is being prepared
        builder: Option<Mutex<RequestBuilder>>,
    }

    impl Http {
        #[instruction(
            name = "Prepare GET Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn prepare_get(url: String) {
            state.builder = Some(Mutex::new(state.client.get(url)));
        }

        #[instruction(
            name = "Prepare HEAD Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn prepare_head(url: String) {
            state.builder = Some(Mutex::new(state.client.head(url)));
        }

        #[instruction(
            name = "Prepare POST Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn prepare_post(url: String) {
            state.builder = Some(Mutex::new(state.client.post(url)));
        }

        #[instruction(
            name = "Prepare PUT Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn prepare_put(url: String) {
            state.builder = Some(Mutex::new(state.client.put(url)));
        }

        #[instruction(
            name = "Prepare PATCH Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn prepare_patch(url: String) {
            state.builder = Some(Mutex::new(state.client.patch(url)));
        }

        #[instruction(
            name = "Prepare DELETE Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn prepare_delete(url: String) {
            state.builder = Some(Mutex::new(state.client.delete(url)));
        }

        #[instruction(
            name = "Add Header to Request",
            flags = InstructionFlags::INFALLIBLE | InstructionFlags::AUTOMATIC,
        )]
        fn add_header(key: String, value: String) {
            if let Some(builder) = state.builder.take() {
                state.builder = Some(Mutex::new(builder.into_inner().header(key, value)));
            } else {
                Err("Trying to add a header without preparing a request first!")?
            }
        }

        #[instruction(
            name = "Add Body to Request",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn add_body(body: String) {
            if let Some(builder) = state.builder.take() {
                state.builder = Some(Mutex::new(builder.into_inner().body(body)));
            } else {
                Err("Trying to add body without preparing a request first!")?
            }
        }

        #[instruction(
            name = "Send Request",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn send() -> #[output(id = "body", name = "Response Body")] String {
            if dry_run {
                return Ok(());
            }

            if let Some(builder) = state.builder.take() {
                let (cl, req) = builder.into_inner().build_split();
                let req = req?;
                let url = req.url().to_string();
                let req_ev = req_to_evidence(&req);
                let res = cl.execute(req)?;

                // Store last request values
                state.last_status = Some(res.status());
                state.last_headers = Some(res.headers().clone());
                let mut body = String::new();
                let res_ev = res_to_evidence(res, &mut body);

                evidence.push(Evidence { label: format!("Request to {url}"), content: EvidenceContent::HttpRequestResponse(req_ev, res_ev) });

                body
            } else {
                Err("Trying to send a request without preparing a request first!")?
            }
        }

        #[instruction(
            name = "Get the Status of the Last Request",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn last_status() -> #[output(id = "status_code", name = "Status Code")] i32 {
            if let Some(status) = state.last_status {
                i32::from(status.as_u16())
            } else {
                Err("Trying to fetch a status without making a request first!")?
            }
        }

        #[instruction(
            name = "Get a Header from the Last Request",
            flags = InstructionFlags::AUTOMATIC,
        )]
        fn last_request_header(key: String) -> #[output(id = "value", name = "Header Value")] String {
            if let Some(headers) = &state.last_headers {
                if let Some(h) = headers.get(&key).cloned() {
                    h.to_str()?.to_string()
                } else {
                    String::new()
                }
            } else {
                Err("Trying to fetch a status without making a request first!")?
            }
        }
    }
}
