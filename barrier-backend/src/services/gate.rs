use log::{debug, error};
use simple_xml_builder::XMLElement;
use tokio::time::{sleep, Duration};

struct XMLParam<'a> {
    name: &'a str,
    value: &'a str,
    attr_type: &'a str,
}

const fn get_params(device_address: &str) -> [XMLParam; 8] {
    [
        XMLParam {
            name: "ComPort",
            value: "2",
            attr_type: "int",
        },
        XMLParam {
            name: "PKUAddress",
            value: "0",
            attr_type: "int",
        },
        XMLParam {
            name: "DeviceAddress",
            value: device_address,
            attr_type: "int",
        },
        XMLParam {
            name: "AggregateAddress",
            value: "1",
            attr_type: "int",
        },
        XMLParam {
            name: "Command",
            value: "0",
            attr_type: "int",
        },
        XMLParam {
            name: "MethodNameForAnswer",
            value: "Result",
            attr_type: "string",
        },
        XMLParam {
            name: "IPSERVER",
            value: "127.0.0.1",
            attr_type: "string",
        },
        XMLParam {
            name: "PORTSERVER",
            value: "8080",
            attr_type: "int",
        },
    ]
}

fn generate_xml(device_address: i32) -> XMLElement {
    let device_address = device_address.to_string();
    let xml_params = get_params(&device_address);

    let mut method_call = XMLElement::new("methodCall");
    let mut method_name = XMLElement::new("methodName");

    method_name.add_text("ControlAccess");

    let mut params = XMLElement::new("params");
    let mut param = XMLElement::new("param");
    let mut value = XMLElement::new("value");
    let mut st = XMLElement::new("struct");

    for p in &xml_params {
        let mut member = XMLElement::new("member");
        let mut name = XMLElement::new("name");
        let mut value = XMLElement::new("value");
        let mut t = XMLElement::new(p.attr_type);

        t.add_text(p.value);

        name.add_text(p.name);
        value.add_child(t);
        member.add_child(name);
        member.add_child(value);
        st.add_child(member);
    }

    value.add_child(st);
    param.add_child(value);
    params.add_child(param);

    method_call.add_child(params);
    method_call.add_child(method_name);

    method_call
}

pub async fn open(server_address: &str, gate: i32, retries: i32) -> Result<(), ()> {
    debug!("try to open {} gate with {} retries", gate, retries);

    let xml = generate_xml(gate);
    let client = reqwest::Client::new();

    let requests = (1..=retries).map(|_| {
        client
            .post(server_address)
            .header("Content-Type", "application/xml")
            .body(format!("{}", xml))
            .send()
    });

    let mut failed_requests = 0;

    for request in requests {
        if !request.await.map_err(|_| ())?.status().is_success() {
            failed_requests += 1;
        }

        sleep(Duration::from_millis(100)).await
    }

    if failed_requests == retries {
        error!("failed to open {} gate", gate);
        Err(())
    } else {
        debug!("relay was opened");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const EXPECTED: &str = r#"<?xml version = "1.0" encoding = "UTF-8"?>
<methodCall>
	<params>
		<param>
			<value>
				<struct>
					<member>
						<name>ComPort</name>
						<value>
							<int>2</int>
						</value>
					</member>
					<member>
						<name>PKUAddress</name>
						<value>
							<int>0</int>
						</value>
					</member>
					<member>
						<name>DeviceAddress</name>
						<value>
							<int>666</int>
						</value>
					</member>
					<member>
						<name>AggregateAddress</name>
						<value>
							<int>1</int>
						</value>
					</member>
					<member>
						<name>Command</name>
						<value>
							<int>0</int>
						</value>
					</member>
					<member>
						<name>MethodNameForAnswer</name>
						<value>
							<string>Result</string>
						</value>
					</member>
					<member>
						<name>IPSERVER</name>
						<value>
							<string>127.0.0.1</string>
						</value>
					</member>
					<member>
						<name>PORTSERVER</name>
						<value>
							<int>8080</int>
						</value>
					</member>
				</struct>
			</value>
		</param>
	</params>
	<methodName>ControlAccess</methodName>
</methodCall>
"#;

    #[test]
    fn check_function() {
        let xml = generate_xml(666);

        assert_eq!(format!("{}", xml), EXPECTED);
    }
}
