use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use regex::Regex;

use serde::Deserialize;
use ndarray::{Array2, Array1};
use std::fs;

// Define the structure for JSON deserialization
#[derive(Deserialize)]
pub struct Layer {
    weights: Vec<Vec<f64>>, // 2D matrix
    biases: Vec<f64>,       // 1D array
}

pub fn relu(x: &Array1<f64>) -> Array1<f64> {
    x.mapv(|v| if v > 0.0 { v } else { 0.0 })
}

pub fn predict(input: Array1<f64>, layers: &Vec<Layer>) -> Array1<f64> {
    let mut activations = input;
    for (i, layer) in layers.iter().enumerate() {
        let weights = Array2::from_shape_vec(
            (layer.weights.len(), layer.weights[0].len()),
            layer.weights.clone().into_iter().flatten().collect(),
        )
        .unwrap();
        let biases = Array1::from_vec(layer.biases.clone());

        // Perform matrix multiplication and add biases
        activations = activations.dot(&weights) + biases;

        // Apply ReLU activation for hidden layers
        if i < layers.len() - 1 {
            activations = relu(&activations);
        }
    }
    activations // Return the final layer's output
}

pub fn load() -> Vec<Layer> {
    // Load JSON file
    let path = "C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/model_weights.json";
    let file_content = fs::read_to_string(path).expect("Failed to read file");
    let layers: Vec<Layer> = serde_json::from_str(&file_content).expect("Failed to parse JSON");
    layers
}


pub fn communicate(server_address: &str, inputs: &Vec<u8>) -> f32 {
    let mut stream = TcpStream::connect(server_address).unwrap();

    // Prepare JSON payload
    let input_data = serde_json::json!({
        "input": inputs,  // Example input data
    });

    let payload = input_data.to_string();

    // Create an HTTP POST request
    let request = format!(
        "POST /predict HTTP/1.1\r\n\
        Host: {}\r\n\
        Content-Type: application/json\r\n\
        Content-Length: {}\r\n\r\n\
        {}",
        server_address, payload.len(), payload
    );

    // Send the request
    if let Err(e) = stream.write_all(request.as_bytes()) {
        eprintln!("Failed to send request: {}", e);
        return 0.0;
    }

    // Read the response
    let mut buffer = String::new();
    if let Err(e) = stream.read_to_string(&mut buffer) {
        eprintln!("Failed to read response: {}", e);
        return 0.0;
    }
    // Blocking on reading the response
    // let mut buffer = [0; 512];  // Buffer to hold the response
    // let bytes_read = stream.read(&mut buffer).unwrap();  // Blocks until data is read

    // Regular expression to extract value within the double square brackets [[value]]
    let re = Regex::new(r"\[\s*\[\s*([\d\.]+)\s*\]\s*\]").unwrap();

    // Search for matches
    // let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    let captures = re.captures(&buffer);
    if let None = captures {
        return 0.0;
    }
    let value = captures.unwrap().get(1).unwrap().as_str().parse::<f32>().unwrap();
    // if let Some(captures) = re.captures(response) {
    //     if let Some(value) = captures.get(1) {
    //         println!("Extracted value: {}", value.as_str());
    //     } else {
    //         println!("No value found in parentheses.");
    //     }
    // } else {
    //     println!("No match found.");
    // }

    // Print the response
    value
}


pub fn open_program() -> (Child, ChildStdin, BufReader<ChildStdout>) {
    // Define the Python executable and script path
    let python_executable = "python";
    let python_script = "C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/predict.py"; // Ensure this script is in your working directory
    let model_path = "C:/Users/adtro/Uni/MatCAD/3r/APC/kaggle/my_model.h5";

    // Start the external program
    let mut child = Command::new(python_executable)
        .arg(&format!("{}", python_script)) // Path to the Python script
        .arg(&format!("{}", model_path)) // Path to the nn model
        .stdin(Stdio::piped()) // Pipe stdin to send input
        .stdout(Stdio::piped()) // Pipe stdout to read output
        .spawn()
        .expect("Failed to start the external program");

    // Get handles to stdin and stdout
    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let child_stdout = child.stdout.take().expect("Failed to open stdout");

    // Wrap stdout in a buffered reader for line-by-line reading
    let mut reader = BufReader::new(child_stdout);

    // Process each line as it's printed
    for line in reader.by_ref().lines() {
        match line {
            Ok(content) => {
                // println!("--------------------{}", content);
                if content.trim() == "isready" { break; }
            },
            Err(e) => { eprintln!("Error reading line: {}", e); break; },
        }
    };
    println!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<");
    let inputs = vec![0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,0,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,1,1,1];
    send_and_receive(&mut child, &mut child_stdin, &mut reader, inputs.clone());
    send_and_receive(&mut child, &mut child_stdin, &mut reader, inputs.clone());
    send_and_receive(&mut child, &mut child_stdin, &mut reader, inputs.clone());
    send_and_receive(&mut child, &mut child_stdin, &mut reader, inputs);
    (child, child_stdin, reader)
}

pub fn send_and_receive(child: &mut Child, child_stdin: &mut ChildStdin, reader: &mut BufReader<ChildStdout>, inputs: Vec<isize>) {
    // let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    // let child_stdout = child.stdout.take().expect("Failed to open stdout");
    // // Wrap stdout in a buffered reader for line-by-line reading
    // let mut reader = BufReader::new(child_stdout);

    let inputs: Vec<String> = inputs.iter().map(|v| v.to_string()).collect();
    // println!("{}", inputs.join(" "));
    // Send input to the external program
    writeln!(child_stdin, "{}", inputs.join(" ")).expect("Failed to write to stdin");
    child_stdin.flush().expect("Failed to flush stdin");

    // Read and print the response
    let mut response = String::new();
    reader.read_line(&mut response).expect("Failed to read from stdout");
    println!("Response: {}", response.trim());
    {
        child
    };
}

pub fn close_connection(child_stdin: ChildStdin, mut child: Child) {
    // Close stdin to signal the end of input
    drop(child_stdin);

    // Wait for the program to finish
    let _ = child.wait().expect("Failed to wait on child process");
}
