use utopia_common::frontend;

use crate::{result_printer, frontend::SockStreamMap, core::Library};

pub async fn send_updated_item(library: &Library, uuid: &String, connections: &mut SockStreamMap) {
	match library.get(uuid) {
		Ok(item) => {
			let details = frontend::CoreEvent::new(frontend::CoreActions::ResponseGameUpdate(item.to_frontend()), None);
			result_printer!(connections.broadcast_stream(details).await, "Failed writing to FE");
		}
		Err(e) => eprintln!("Utopia Error: {}", e),
	};
}
