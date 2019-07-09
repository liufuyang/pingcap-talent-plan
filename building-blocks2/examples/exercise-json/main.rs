use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::Cursor;

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    x: i32,
    y: i32,
}

fn main() {
    let point = Move { x: 1, y: 2 };
    let point2 = Move { x: 3, y: 4 };

    // Json
    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&point).unwrap();
    // Move serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);
    // Convert the JSON string back to a Move.
    let deserialized: Move = serde_json::from_str(&serialized).unwrap();
    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized from String = {:?}", deserialized);

    // to File
    let file = File::create("test.json").unwrap();
    let _result = serde_json::to_writer(file, &point);

    let file = File::open("test.json").unwrap();
    let deserialized: Move = serde_json::from_reader(file).unwrap();
    println!("deserialized from File = {:?}", deserialized);

    // Vec<u8>
    let buffer = serde_json::to_vec(&point).unwrap();
    println!("serialized buffer = {:?}", buffer);
    let deserialized: Move = serde_json::from_reader(&buffer[..]).unwrap();
    println!("deserialized from String = {:?}", deserialized);

    // RSON
    let serialized = ron::ser::to_string(&point).unwrap().into_bytes();
    println!(
        "serialized RSON buffer String = {:?}",
        String::from_utf8(serialized.clone()).unwrap()
    );
    let deserialized: Move = ron::de::from_reader(&serialized[..]).unwrap();
    println!("deserialized from RSON String = {:?}", deserialized);

    // BSON
    let mut file = File::create("test.bson").unwrap();
    let serialized = bson::to_bson(&point).unwrap();
    let serialized2 = bson::to_bson(&point2).unwrap();

    if let bson::Bson::Document(document) = serialized {
        println!("{:?}", document);
        bson::encode_document(&mut file, &document);
    }
    if let bson::Bson::Document(document) = serialized2 {
        println!("{:?}", document);
        bson::encode_document(&mut file, &document);
    }

    let mut file = File::open("test.bson").unwrap();

    let doc = bson::decode_document(&mut file).unwrap();
    let deserialized: Move = bson::from_bson(bson::Bson::Document(doc)).unwrap();
    println!("deserialized from BSON file = {:?}", deserialized);

    let doc = bson::decode_document(&mut file).unwrap();
    let deserialized: Move = bson::from_bson(bson::Bson::Document(doc)).unwrap();
    println!("deserialized from BSON file = {:?}", deserialized);

    // BSON Vec
    let mut file = Vec::new();
    let serialized = bson::to_bson(&point).unwrap();
    let serialized2 = bson::to_bson(&point2).unwrap();

    if let bson::Bson::Document(document) = serialized {
        println!("{:?}", document);
        bson::encode_document(&mut file, &document);
    }
    if let bson::Bson::Document(document) = serialized2 {
        println!("{:?}", document);
        bson::encode_document(&mut file, &document);
    }

    let mut file: Cursor<Vec<u8>> = Cursor::new(file);
    let doc = bson::decode_document(&mut file).unwrap();
    let deserialized: Move = bson::from_bson(bson::Bson::Document(doc)).unwrap();
    println!("deserialized from BSON Vec = {:?}", deserialized);

    let doc = bson::decode_document(&mut file).unwrap();
    let deserialized: Move = bson::from_bson(bson::Bson::Document(doc)).unwrap();
    println!("deserialized from BSON Vec = {:?}", deserialized);
}

// Output:
// serialized = {"x":1,"y":2}
// deserialized from String = Move { x: 1, y: 2 }
// deserialized from File = Move { x: 1, y: 2 }
// serialized buffer = [123, 34, 120, 34, 58, 49, 44, 34, 121, 34, 58, 50, 125]
// deserialized from String = Move { x: 1, y: 2 }
// serialized RSON buffer String = "(x:1,y:2,)"
// deserialized from RSON String = Move { x: 1, y: 2 }
// OrderedDocument({"x": I32(1), "y": I32(2)})
// OrderedDocument({"x": I32(3), "y": I32(4)})
// deserialized from BSON file = Move { x: 1, y: 2 }
// deserialized from BSON file = Move { x: 3, y: 4 }
// OrderedDocument({"x": I32(1), "y": I32(2)})
// OrderedDocument({"x": I32(3), "y": I32(4)})
// deserialized from BSON Vec = Move { x: 1, y: 2 }
// deserialized from BSON Vec = Move { x: 3, y: 4 }
