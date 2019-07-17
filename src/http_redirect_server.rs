use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::Shutdown;
use std::error::Error;
use std::str;

pub fn start_redirect_server(port: &String, redirect_url: &String) -> Result<(), Box<dyn Error>> {
    let mut redirect_response = String::from("HTTP/1.1 302 Found\r\nLocation: ");
    redirect_response.push_str(redirect_url);
    redirect_response.push_str(String::from("\r\n").as_str());
    redirect_response.push_str(String::from("Cache-Control: max-age=3600\r\n").as_str());
    redirect_response.push_str(String::from("Origin: iron-spider\r\n").as_str());
    let mut ip = "0.0.0.0:".to_owned();
    ip.push_str(&port);
    info!("Binding redirect server to {}", ip);
    let listener = TcpListener::bind(ip)?;
    info!("Bound to ip");
    for stream in listener.incoming() {
        debug!("Got tcp request");
        let mut s = stream?;

        let mut data = [0 as u8; 1024];
        match s.read(&mut data) {
            Ok(_) => {
                debug!("Data Received: {:#?}", str::from_utf8(&data)?);
                // return Redirect response
            }
            Err(_) => {}
        } // this thing is the body of the while loop

        s.shutdown(Shutdown::Read)?;
        debug!("returning response: {:#?}", redirect_response);
        s.write_all(redirect_response.as_bytes())?;
        s.flush()?;
        s.shutdown(Shutdown::Write)?;
        debug!("Done");
    }
    Ok(())
}
