/*
 * rumtk attempts to implement HL7 and medical protocols for interoperability in medicine.
 * This toolkit aims to be reliable, simple, performant, and standards compliant.
 * Copyright (C) 2024  Luis M. Santos, M.D.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 */

//#![feature(unboxed_closures)]
#![feature(inherent_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![feature(buf_read_has_data_left)]

pub mod cache;
pub mod cli;
pub mod core;
pub mod json;
pub mod log;
pub mod maths;
pub mod net;
pub mod queue;
pub mod scripting;
pub mod search;
pub mod strings;
pub mod threading;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::RUMCache;
    use crate::search::rumtk_search::*;
    use crate::strings::{RUMArrayConversions, RUMString, RUMStringConversions, StringUtils};
    use compact_str::{format_compact, CompactString};
    use serde::{Deserialize, Serialize};
    use std::future::IntoFuture;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[test]
    fn test_is_escaped_str() {
        let input = "\r\n\'\\\"";
        let expected = false;
        let result = strings::is_escaped_str(input);
        println!("Input: {} Expected: {} Got: {}", input, expected, result);
        assert_eq!(
            expected, result,
            "Incorrect detection of unescaped string as escaped!!"
        );
        println!("Passed!")
    }

    #[test]
    fn test_escaping_control() {
        let input = "\r\n\'\"\\";
        let expected = "\\r\\n\\'\\\"\\\\";
        let result = strings::escape(input);
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            expected,
            result.as_str()
        );
        assert_eq!(expected, result, "Incorrect string escaping!");
        println!("Passed!")
    }

    #[test]
    fn test_escaping_unicode() {
        let input = "❤";
        let expected = "\\u2764";
        let result = strings::escape(input);
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            expected,
            result.as_str()
        );
        assert_eq!(expected, result, "Incorrect string escaping!");
        println!("Passed!")
    }

    #[test]
    fn test_unescaping_unicode() {
        let input = "❤";
        let escaped = strings::escape(input);
        let expected = "❤";
        let result = RUMString::from_utf8(strings::unescape(escaped.as_str()).unwrap()).unwrap();
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            expected,
            result.as_str()
        );
        assert_eq!(expected, result.as_str(), "Incorrect string unescaping!");
        println!("Passed!")
    }

    #[test]
    fn test_unescaping_string() {
        let input = "I \\u2764 my wife!";
        let expected = "I ❤ my wife!";
        let result = strings::unescape_string(input).unwrap();
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            expected,
            result.as_str()
        );
        assert_eq!(expected, result.as_str(), "Incorrect string unescaping!");
        println!("Passed!")
    }

    #[test]
    fn test_is_escaped_string() {
        let input = "I \\u2764 my wife!";
        let expected = true;
        let result = strings::is_escaped_str(input);
        println!("Input: {} Expected: {} Got: {}", input, expected, result);
        assert_eq!(
            expected, result,
            "Escaped string detected as unescaped string!"
        );
        println!("Passed!")
    }

    #[test]
    fn test_is_unescaped_string() {
        let input = "I ❤ my wife!";
        let expected = false;
        let result = strings::is_escaped_str(input);
        println!("Input: {} Expected: {} Got: {}", input, expected, result);
        assert_eq!(
            expected, result,
            "Unescaped string detected as escaped string!"
        );
        println!("Passed!")
    }

    #[test]
    fn test_unique_string() {
        let input = "I❤mywife!";
        assert!(input.is_unique(), "String was not detected as unique.");
    }

    #[test]
    fn test_non_unique_string() {
        let input = "I❤❤mywife!";
        assert!(!input.is_unique(), "String was detected as unique.");
    }

    #[test]
    fn test_escaping_string() {
        let input = "I ❤ my wife!";
        let expected = "I \\u2764 my wife!";
        let result = strings::escape(input);
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            expected,
            result.as_str()
        );
        assert_eq!(expected, result.as_str(), "Incorrect string escaping!");
        println!("Passed!")
    }

    #[test]
    fn test_autodecode_utf8() {
        let input = "I ❤ my wife!";
        let result = strings::try_decode(input.as_bytes());
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            input,
            result.as_str()
        );
        assert_eq!(input, result, "Incorrect string decoding!");
        println!("Passed!")
    }

    #[test]
    fn test_autodecode_other() {
        //TODO: Need an example of other encoding texts.
        let input = "I ❤ my wife!";
        let result = input;
        println!("Input: {} Expected: {} Got: {}", input, input, result);
        assert_eq!(input, result, "Incorrect string decoding!");
        println!("Passed!")
    }

    #[test]
    fn test_decode() {
        let input = "I ❤ my wife!";
        let result = strings::try_decode_with(input.as_bytes(), "utf-8");
        println!(
            "Input: {} Expected: {} Got: {}",
            input,
            input,
            result.as_str()
        );
        assert_eq!(input, result, "Incorrect string decoding!");
        println!("Passed!")
    }

    #[test]
    fn test_rumcache_insertion() {
        let mut cache: RUMCache<&str, CompactString> = RUMCache::with_capacity(5);
        cache.insert("❤", CompactString::from("I ❤ my wife!"));
        println!("Contents: {:#?}", &cache);
        assert_eq!(cache.len(), 1, "Incorrect number of items in cache!");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_letters() {
        let input = "Hello World!";
        let expr = r"\w";
        let result = string_search(input, expr, "");
        let expected: RUMString = RUMString::from("HelloWorld");
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            input, expected, result
        );
        assert_eq!(expected, result, "String search results mismatch");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_words() {
        let input = "Hello World!";
        let expr = r"\w+";
        let result = string_search(input, expr, " ");
        let expected: RUMString = RUMString::from("Hello World");
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            input, expected, result
        );
        assert_eq!(expected, result, "String search results mismatch");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_named_groups() {
        let input = "Hello World!";
        let expr = r"(?<hello>\w{5}) (?<world>\w{5})";
        let result = string_search_named_captures(input, expr, "");
        let expected: RUMString = RUMString::from("World");
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            input, expected, result
        );
        assert_eq!(expected, result["world"], "String search results mismatch");
        println!("Passed!")
    }

    #[test]
    fn test_search_string_all_groups() {
        let input = "Hello World!";
        let expr = r"(?<hello>\w{5}) (?<world>\w{5})";
        let result = string_search_all_captures(input, expr, "");
        let expected: Vec<&str> = vec!["Hello", "World"];
        println!(
            "Input: {:?} Expected: {:?} Got: {:?}",
            input, expected, result
        );
        assert_eq!(expected, result, "String search results mismatch");
        println!("Passed!")
    }

    ///////////////////////////////////Threading Tests/////////////////////////////////////////////////
    #[test]
    fn test_default_num_threads() {
        use num_cpus;
        let threads = threading::threading_functions::get_default_system_thread_count();
        assert_eq!(
            threads >= num_cpus::get(),
            true,
            "Default thread count is incorrect! We got {}, but expected {}!",
            threads,
            num_cpus::get()
        );
    }

    #[test]
    fn test_execute_job() {
        let rt = rumtk_init_threads!();
        let expected = vec![1, 2, 3];
        let task_processor = async |args: &SafeTaskArgs<i32>| -> TaskResult<i32> {
            let owned_args = Arc::clone(args);
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let mut results = TaskItems::<i32>::with_capacity(locked_args.len());
            print!("Contents: ");
            for arg in locked_args.iter() {
                results.push(arg.clone());
                println!("{} ", &arg);
            }
            Ok(results)
        };
        let locked_args = RwLock::new(expected.clone());
        let task_args = SafeTaskArgs::<i32>::new(locked_args);
        let task_result = rumtk_wait_on_task!(rt, task_processor, &task_args);
        let result = task_result.unwrap();
        assert_eq!(&result, &expected, "{}", format_compact!("Task processing returned a different result than expected! Expected {:?} \nResults {:?}", &expected, &result));
    }

    #[test]
    fn test_execute_job_macros() {
        let rt = rumtk_init_threads!();
        let expected = vec![1, 2, 3];
        let task_processor = async |args: &SafeTaskArgs<i32>| -> TaskResult<i32> {
            let owned_args = Arc::clone(args);
            let lock_future = owned_args.read();
            let locked_args = lock_future.await;
            let mut results = TaskItems::<i32>::with_capacity(locked_args.len());
            print!("Contents: ");
            for arg in locked_args.iter() {
                results.push(arg.clone());
                println!("{} ", &arg);
            }
            Ok(results)
        };
        let task_args = rumtk_create_task_args!(1, 2, 3);
        let task_result = rumtk_wait_on_task!(rt, task_processor, &task_args);
        let result = task_result.unwrap();
        assert_eq!(&result, &expected, "{}", format_compact!("Task processing returned a different result than expected! Expected {:?} \nResults {:?}", &expected, &result));
    }

    #[test]
    fn test_execute_job_macros_one_line() {
        let rt = rumtk_init_threads!();
        let expected = vec![1, 2, 3];
        let result = rumtk_exec_task!(
            async |args: &SafeTaskArgs<i32>| -> TaskResult<i32> {
                let owned_args = Arc::clone(args);
                let lock_future = owned_args.read();
                let locked_args = lock_future.await;
                let mut results = TaskItems::<i32>::with_capacity(locked_args.len());
                print!("Contents: ");
                for arg in locked_args.iter() {
                    results.push(arg.clone());
                    println!("{} ", &arg);
                }
                Ok(results)
            },
            vec![1, 2, 3]
        )
        .unwrap();
        assert_eq!(&result, &expected, "{}", format_compact!("Task processing returned a different result than expected! Expected {:?} \nResults {:?}", &expected, &result));
    }

    #[test]
    fn test_clamp_index_positive_index() {
        let values = vec![1, 2, 3, 4];
        let given_index = 3isize;
        let max_size = values.len() as isize;
        let index = clamp_index(&given_index, &max_size).unwrap();
        assert_eq!(
            index, 3,
            "Index mismatch! Requested index {} but got {}",
            &given_index, &index
        );
        assert_eq!(
            values[index], 4,
            "Value mismatch! Expected {} but got {}",
            &values[3], &values[index]
        );
    }

    #[test]
    fn test_clamp_index_reverse_index() {
        let values = vec![1, 2, 3, 4];
        let given_index = -1isize;
        let max_size = values.len() as isize;
        let index = clamp_index(&given_index, &max_size).unwrap();
        assert_eq!(
            index, 4,
            "Index mismatch! Requested index {} but got {}",
            &given_index, &index
        );
        assert_eq!(
            values[index - 1],
            4,
            "Value mismatch! Expected {} but got {}",
            &values[3],
            &values[index]
        );
    }

    ///////////////////////////////////Queue Tests/////////////////////////////////////////////////
    use crate::cli::cli_utils::print_license_notice;
    use crate::core::clamp_index;
    use crate::net::tcp::LOCALHOST;
    use crate::threading::thread_primitives::{SafeTaskArgs, TaskItems, TaskResult};
    use crate::threading::threading_functions::sleep;
    use queue::queue::*;

    #[test]
    fn test_queue_data() {
        let expected = vec![
            RUMString::from("Hello"),
            RUMString::from("World!"),
            RUMString::from("Overcast"),
            RUMString::from("and"),
            RUMString::from("Sad"),
        ];
        let mut queue = TaskQueue::<RUMString>::new(&5).unwrap();
        let locked_args = RwLock::new(expected.clone());
        let task_args = SafeTaskArgs::<RUMString>::new(locked_args);
        let processor = rumtk_create_task!(
            async |args: &SafeTaskArgs<RUMString>| -> TaskResult<RUMString> {
                let owned_args = Arc::clone(args);
                let lock_future = owned_args.read();
                let locked_args = lock_future.await;
                let mut results = TaskItems::<RUMString>::with_capacity(locked_args.len());
                print!("Contents: ");
                for arg in locked_args.iter() {
                    print!("{} ", &arg);
                    results.push(RUMString::new(arg));
                }
                Ok(results)
            },
            task_args
        );
        queue.add_task::<_>(processor);
        let results = queue.wait();
        let mut result_data = Vec::<RUMString>::with_capacity(5);
        for r in results {
            for v in r.unwrap().iter() {
                result_data.push(v.clone());
            }
        }
        assert_eq!(result_data, expected, "Results do not match expected!");
    }

    ///////////////////////////////////Net Tests/////////////////////////////////////////////////
    #[test]
    fn test_server_start() {
        let mut server = match rumtk_create_server!("localhost", 0) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        }
    }

    #[test]
    fn test_server_send() {
        let msg = RUMString::from("Hello World!");
        let mut server = match rumtk_create_server!(LOCALHOST, 0, 1) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        };
        let address_info = server.get_address_info().unwrap();
        let (ip, port) = rumtk_get_ip_port!(address_info);
        println!("Sleeping");
        rumtk_sleep!(1);
        let mut client = match rumtk_connect!(port) {
            Ok(client) => client,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        let client_id = client.get_address().unwrap();
        rumtk_sleep!(1);
        match server.send(&client_id, &msg.to_raw()) {
            Ok(_) => (),
            Err(e) => panic!("Server failed to send message because {}", e),
        };
        rumtk_sleep!(1);
        let received_message = client.receive().unwrap();
        assert_eq!(
            &msg.to_raw(),
            &received_message,
            "{}",
            format_compact!(
                "Received message does not match sent message by server {:?}",
                &received_message
            )
        );
    }

    #[test]
    fn test_server_receive() {
        let msg = RUMString::from("Hello World!");
        let mut server = match rumtk_create_server!(LOCALHOST, 0) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        };
        let address_info = server.get_address_info().unwrap();
        let (ip, port) = rumtk_get_ip_port!(address_info);
        println!("Sleeping");
        rumtk_sleep!(1);
        let mut client = match rumtk_connect!(port) {
            Ok(client) => client,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match client.send(&msg.to_raw()) {
            Ok(_) => (),
            Err(e) => panic!("Failed to send message because {}", e),
        };
        rumtk_sleep!(1);
        let client_id = client.get_address().expect("Failed to get client id");
        let incoming_message = server.receive(&client_id).unwrap().to_rumstring();
        println!("Received message => {:?}", &incoming_message);
        assert_eq!(&incoming_message, msg, "Received message corruption!");
    }

    #[test]
    fn test_server_get_clients() {
        let mut server = match rumtk_create_server!(LOCALHOST, 0) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        };
        let address_info = server.get_address_info().unwrap();
        let (ip, port) = rumtk_get_ip_port!(address_info);
        println!("Sleeping");
        rumtk_sleep!(1);
        let mut client = match rumtk_connect!(port) {
            Ok(client) => client,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        rumtk_sleep!(1);
        let expected_client_id = client.get_address().expect("Failed to get client id");
        let clients = server.get_client_ids();
        let incoming_client_id = clients.get(0).expect("Expected client to have connected!");
        println!("Connected client id => {}", &incoming_client_id);
        assert_eq!(
            &incoming_client_id, &expected_client_id,
            "Connected client does not match the connecting client! Client id => {}",
            &incoming_client_id
        );
    }

    #[test]
    fn test_server_stop() {
        let msg = RUMString::from("Hello World!");
        let mut server = match rumtk_create_server!("localhost", 0) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        };
        println!("Sleeping");
        rumtk_sleep!(1);
        match server.stop() {
            Ok(_) => (),
            Err(e) => panic!("Failed to stop server because {}", e),
        };
    }

    #[test]
    fn test_server_get_address_info() {
        let msg = RUMString::from("Hello World!");
        let mut server = match rumtk_create_server!("localhost", 0) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        };
        println!("Sleeping");
        rumtk_sleep!(1);
        match server.get_address_info() {
            Some(addr) => println!("Server address info => {}", addr),
            None => panic!("No address. Perhaps the server was never initialized?"),
        };
    }

    #[test]
    fn test_client_send() {
        let msg = RUMString::from("Hello World!");
        let mut server = match rumtk_create_server!(LOCALHOST, 0) {
            Ok(server) => server,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        match server.start(false) {
            Ok(_) => (),
            Err(e) => panic!("Failed to start server because {}", e),
        };
        let address_info = server.get_address_info().unwrap();
        let (ip, port) = rumtk_get_ip_port!(address_info);
        println!("Sleeping");
        rumtk_sleep!(1);
        let mut client = match rumtk_connect!(port) {
            Ok(client) => client,
            Err(e) => panic!("Failed to create server because {}", e),
        };
        rumtk_sleep!(2);
        match client.send(&msg.to_raw()) {
            Ok(_) => (),
            Err(e) => panic!("Failed to send message because {}", e),
        };
        rumtk_sleep!(1);
        let clients = server.get_client_ids();
        let incoming_client_id = clients.first().expect("Expected client to have connected!");
        let mut received_message = server.receive(incoming_client_id).unwrap();
        if received_message.is_empty() {
            rumtk_sleep!(1);
            received_message = server.receive(incoming_client_id).unwrap();
        }
        assert_eq!(
            &msg.to_raw(),
            &received_message,
            "{}",
            format_compact!(
                "Received message does not match sent message by client {:?}",
                &received_message
            )
        );
    }

    ////////////////////////////JSON Tests/////////////////////////////////

    #[test]
    fn test_serialize_json() {
        #[derive(Serialize)]
        struct MyStruct {
            hello: RUMString,
        }

        let hw = MyStruct {
            hello: RUMString::from("World"),
        };
        let hw_str = rumtk_serialize!(&hw, true).unwrap();

        assert!(
            !hw_str.is_empty(),
            "Empty JSON string generated from the test struct!"
        );
    }

    #[test]
    fn test_deserialize_json() {
        #[derive(Serialize, Deserialize, PartialEq)]
        struct MyStruct {
            hello: RUMString,
        }

        let hw = MyStruct {
            hello: RUMString::from("World"),
        };
        let hw_str = rumtk_serialize!(&hw, true).unwrap();
        let new_hw: MyStruct = rumtk_deserialize!(&hw_str).unwrap();

        assert!(
            new_hw == hw,
            "Deserialized JSON does not match the expected value!"
        );
    }

    /*
    #[test]
    fn test_escape_unescape_json() {
        #[derive(Serialize, Deserialize, PartialEq)]
        struct MyStruct {
            hello: RUMString,
        }

        let hw = MyStruct {
            hello: RUMString::from("World"),
        };

        let hw_str = rumtk_serialize!(&hw, true).unwrap();
        let hw_escaped_str = strings::basic_escape(&hw_str);
        println!("Escaped => {}", hw_escaped_str);

        let hw_unescaped_str = strings::unescape_string(&hw_escaped_str).unwrap();
        println!("Unescaped => {}", hw_unescaped_str);
        assert_eq!(
            hw_str.to_rumstring(),
            hw_unescaped_str.to_rumstring(),
            "Unescaped serialized JSON mismatch!"
        );

        let new_hw: MyStruct = rumtk_deserialize!(&hw_unescaped_str).unwrap();

        assert!(
            new_hw == hw,
            "Deserialized JSON does not match the expected value!"
        );
    }
    */

    ////////////////////////////CLI Tests/////////////////////////////////

    #[test]
    fn test_print_license_notice() {
        print_license_notice("RUMTK", "2025", &vec!["Luis M. Santos, M.D."]);
    }

    //////////////////////////////////////////////////////////////////////////////////////////////
}
