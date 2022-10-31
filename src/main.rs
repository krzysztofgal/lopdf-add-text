fn main() {
    // create();
    modify();
}

use lopdf::dictionary;
use lopdf::{
    content::{Content, Operation},
    Dictionary, Document, Object, ObjectId,
};

fn modify() {
    let mut doc = Document::load("example.pdf").unwrap();

    let mut page_id = (0, 0);
    for (_page_num, id) in doc.get_pages() {
        page_id = id;
        break;
    }

    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });

    let mut new_font_name: Option<String> = None;
    let mut fonts_resource_id = None;

    let resources = doc.get_or_create_resources(page_id).unwrap();
    let d = resources.as_dict_mut().unwrap();
    match d.get_mut(b"Font") {
        Ok(Object::Reference(id)) => {
            dbg!("Ref", &id);
            fonts_resource_id = Some(*id);
        }
        Ok(Object::Dictionary(dict)) => {
            dbg!("Dict", &dict);
            new_font_name = add_font_to_resources(dict, font_id);
        }
        unhandled => {
            dbg!("Unhandled: {:?}", &unhandled);
        }
    };

    if let Some(id) = fonts_resource_id {
        if let Ok(dict) = doc.get_dictionary_mut(id) {
            new_font_name = add_font_to_resources(dict, font_id);
            dbg!(&new_font_name);
        }
    }

    if let Some(f_name) = new_font_name {
        let f_name = f_name.as_str();

        let content = Content {
            operations: vec![
                operand("BT"),
                operand("q"),
                operation("rg", [0, 0, 0]),
                operation("Tr", [0]),
                Operation::new("Tf", vec![f_name.into(), 48.into()]),
                operation("Td", [100, 200]),
                //operation("Tm", [1, 0, 0, 1, 100, 200]),
                operation("Tj", [Object::string_literal("Hello World!")]),
                // next
                Operation::new("Tf", vec![f_name.into(), 16.into()]),
                operation("Td", [0, -18]),
                operation("Tj", [Object::string_literal("latin: żółć")]),
                operand("Q"),
                operand("ET"),
            ],
        };

        doc.add_to_page_content(page_id, content).unwrap();

        doc.save("modified.pdf").unwrap();
    } else {
        println!("Unmodified");
    }
}

fn operation<N: std::fmt::Display, OP: Into<lopdf::Object>>(
    name: N,
    ops: impl IntoIterator<Item = OP>,
) -> Operation {
    Operation {
        operator: name.to_string(),
        operands: ops.into_iter().map(|v| v.into()).collect(),
    }
}

fn operand<N: std::fmt::Display>(name: N) -> Operation {
    Operation {
        operator: name.to_string(),
        operands: Vec::new(),
    }
}

fn add_font_to_resources(dict: &mut Dictionary, font_id: ObjectId) -> Option<String> {
    let mut last_font = None;
    dict.iter().for_each(|v| {
        last_font = Some(v.0);
    });

    if let Some(f_name) = last_font {
        let mut new_name = f_name.clone();
        if let Some(mut last_byte) = new_name.pop() {
            last_byte = last_byte + 1;
            new_name.push(last_byte);
        }

        dict.set(new_name.clone(), font_id);

        Some(String::from_utf8_lossy(&new_name).to_string())
    } else {
        None
    }
}
