//! The shipped Terror bell: one seamless clip whose nodes cannot bind as map spaces.
use cuba_libre_unfolded::Game;
use serde_json::Value;

const GLB: &[u8] = include_bytes!("../art/atlas/atlas-terror-token.glb");

fn u32_at(at: usize) -> usize {
    u32::from_le_bytes(GLB[at..at + 4].try_into().unwrap()) as usize
}

/// The JSON and binary chunks of the GLB container.
fn chunks() -> (Value, &'static [u8]) {
    assert_eq!(&GLB[0..4], b"glTF");
    assert_eq!(u32_at(4), 2, "glTF 2.0");
    let json = u32_at(12);
    assert_eq!(&GLB[16..20], b"JSON");
    let gltf = serde_json::from_slice(&GLB[20..20 + json]).unwrap();
    let bin = 20 + json;
    assert_eq!(&GLB[bin + 4..bin + 8], b"BIN\0");
    (gltf, &GLB[bin + 8..bin + 8 + u32_at(bin)])
}

/// An accessor's float components and its width (1 for times, 4 for rotations).
fn floats(gltf: &Value, bin: &[u8], accessor: &Value) -> (usize, Vec<f32>) {
    let accessor = &gltf["accessors"][accessor.as_u64().unwrap() as usize];
    assert_eq!(accessor["componentType"], 5126, "float components");
    let width = match accessor["type"].as_str().unwrap() {
        "SCALAR" => 1,
        "VEC3" => 3,
        "VEC4" => 4,
        other => panic!("unexpected accessor type {other}"),
    };
    let view = &gltf["bufferViews"][accessor["bufferView"].as_u64().unwrap() as usize];
    assert!(view.get("byteStride").is_none(), "tightly packed");
    let start = (view["byteOffset"].as_u64().unwrap_or(0)
        + accessor["byteOffset"].as_u64().unwrap_or(0)) as usize;
    let length = accessor["count"].as_u64().unwrap() as usize * width * 4;
    let values = bin[start..start + length]
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    (width, values)
}

#[test]
fn terror_bell_is_one_clip_whose_last_key_repeats_its_first() {
    let (gltf, bin) = chunks();
    let animations = gltf["animations"].as_array().unwrap();
    assert_eq!(animations.len(), 1, "every bell plays one clip");
    let channels = animations[0]["channels"].as_array().unwrap();
    assert!(!channels.is_empty());
    for channel in channels {
        let node = &gltf["nodes"][channel["target"]["node"].as_u64().unwrap() as usize]["name"];
        let sampler = &animations[0]["samplers"][channel["sampler"].as_u64().unwrap() as usize];
        assert_eq!(sampler["interpolation"], "LINEAR", "{node}");
        let (_, times) = floats(&gltf, bin, &sampler["input"]);
        assert_eq!(times.first(), Some(&0.0), "{node} starts the loop");
        assert!(
            (times.last().unwrap() - 3.6).abs() < 1e-6,
            "{node} ends exactly one 3.6 s loop later, with the wrap key: {:?}",
            times.last()
        );
        let (width, values) = floats(&gltf, bin, &sampler["output"]);
        assert_eq!(values.len(), times.len() * width);
        assert_eq!(
            values[..width],
            values[values.len() - width..],
            "{node} {} wraps without a jump",
            channel["target"]["path"]
        );
    }
}

#[test]
fn terror_bell_node_names_never_bind_as_map_spaces() {
    let (gltf, _) = chunks();
    let spaces: Vec<_> = Game::prepared()
        .observe()
        .spaces
        .into_iter()
        .map(|s| s.id)
        .collect();
    for node in gltf["nodes"].as_array().unwrap() {
        let name = node["name"].as_str().unwrap_or_default();
        assert!(
            !spaces.iter().any(|id| id == name),
            "region binding would claim bell node {name}"
        );
    }
}
