//! The shipped Terror bell: one seamless clip, and no glTF name that region binding
//! could claim as a map space. The GLB is read here independently of Bevy's loader.
use cuba_libre_unfolded::Game;
use serde_json::Value;

const GLB: &[u8] = include_bytes!("../../art/atlas/atlas-terror-token.glb");
/// The authored loop length, in seconds.
const LOOP_SECONDS: f32 = 3.6;
/// glTF accessor component type for 32-bit floats.
const GLTF_FLOAT: u64 = 5126;

fn u32_at(at: usize) -> usize {
    let bytes = GLB.get(at..at + 4).expect("GLB truncated");
    u32::from_le_bytes(bytes.try_into().unwrap()) as usize
}

fn index(value: &Value) -> usize {
    value.as_u64().expect("glTF index") as usize
}

/// The JSON and binary chunks of the GLB container.
fn chunks() -> (Value, &'static [u8]) {
    assert_eq!(&GLB[0..4], b"glTF");
    assert_eq!(u32_at(4), 2, "glTF 2.0");
    assert_eq!(u32_at(8), GLB.len(), "header length matches the file");
    let json = u32_at(12);
    assert_eq!(&GLB[16..20], b"JSON");
    let gltf = serde_json::from_slice(&GLB[20..20 + json]).unwrap();
    let bin = 20 + json;
    assert_eq!(&GLB[bin + 4..bin + 8], b"BIN\0");
    let data = GLB
        .get(bin + 8..bin + 8 + u32_at(bin))
        .expect("BIN chunk truncated");
    (gltf, data)
}

/// An accessor's float components, which must come `width` to an element.
fn floats(gltf: &Value, bin: &[u8], accessor: &Value, width: usize) -> Vec<f32> {
    let accessor = &gltf["accessors"][index(accessor)];
    assert_eq!(accessor["componentType"], GLTF_FLOAT, "float components");
    let actual = match accessor["type"].as_str().unwrap() {
        "SCALAR" => 1,
        "VEC3" => 3,
        "VEC4" => 4,
        other => panic!("unexpected accessor type {other}"),
    };
    assert_eq!(actual, width, "accessor width");
    let view = &gltf["bufferViews"][index(&accessor["bufferView"])];
    assert!(view.get("byteStride").is_none(), "tightly packed");
    let start = (view["byteOffset"].as_u64().unwrap_or(0)
        + accessor["byteOffset"].as_u64().unwrap_or(0)) as usize;
    let length = index(&accessor["count"]) * width * 4;
    bin.get(start..start + length)
        .expect("accessor outside the BIN chunk")
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}

#[test]
fn terror_bell_is_one_clip_whose_last_key_repeats_its_first() {
    let (gltf, bin) = chunks();
    let animations = gltf["animations"].as_array().unwrap();
    assert_eq!(animations.len(), 1, "every bell plays one clip");
    let clip = &animations[0];
    let channels = clip["channels"].as_array().unwrap();
    assert!(!channels.is_empty());
    for channel in channels {
        let target = &channel["target"];
        let node = &gltf["nodes"][index(&target["node"])]["name"];
        let sampler = &clip["samplers"][index(&channel["sampler"])];
        assert_eq!(sampler["interpolation"], "LINEAR", "{node}");
        let times = floats(&gltf, bin, &sampler["input"], 1);
        assert_eq!(times.first(), Some(&0.0), "{node} starts the loop");
        let end = *times.last().unwrap();
        assert!(
            (end - LOOP_SECONDS).abs() < 1e-6,
            "{node} ends exactly one loop later, with the wrap key: {end}"
        );
        let width = match target["path"].as_str().unwrap() {
            "rotation" => 4,
            "translation" | "scale" => 3,
            other => panic!("{node} animates unsupported {other}"),
        };
        let values = floats(&gltf, bin, &sampler["output"], width);
        assert_eq!(values.len(), times.len() * width);
        assert_eq!(
            values[..width],
            values[values.len() - width..],
            "{node} {} wraps without a jump",
            target["path"]
        );
    }
}

#[test]
fn terror_bell_names_never_bind_as_map_spaces() {
    let (gltf, _) = chunks();
    let spaces: Vec<_> = Game::prepared()
        .observe()
        .spaces
        .into_iter()
        .map(|s| s.id)
        .collect();
    // Bevy names spawned entities after scenes, nodes, meshes and
    // `{mesh}.{material}` pairs; the app's region binding matches any Name.
    let named = |key: &str| {
        gltf[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|item| item["name"].as_str())
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let meshes = named("meshes");
    let mut names = [named("scenes"), named("nodes"), meshes.clone()].concat();
    for mesh in &meshes {
        names.extend(named("materials").iter().map(|m| format!("{mesh}.{m}")));
    }
    assert!(!names.is_empty());
    for name in &names {
        assert!(
            !spaces.contains(name),
            "region binding would claim bell entity {name}"
        );
    }
}
