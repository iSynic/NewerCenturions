use centurions_content::{Content, audio, digest, graphics, read_resource};
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn run(content: &Content, root: &Path) -> Value {
    let resources: Vec<Value> = content
        .resources
        .iter()
        .map(|resource| {
            let mut row = json!({"source":resource.original,"type":resource.r#type,"id":resource.id,
            "source_sha256":resource.sha256,"size":resource.size,"status":"inventoried"});
            let kind = resource.r#type.as_str();
            if !matches!(
                kind,
                "PICT"
                    | "cicn"
                    | "ppat"
                    | "snd "
                    | "ICON"
                    | "CURS"
                    | "crsr"
                    | "icl8"
                    | "icl4"
                    | "ics8"
                    | "ics4"
            ) {
                return row;
            }
            let decoded = read_resource(root, resource).and_then(|bytes| {
                if kind == "snd " {
                    return audio::decode_snd(&bytes).map(|sample| {
                        json!({
                    "kind":"audio","channels":sample.channels,"rate_fixed":sample.rate_fixed,
                    "bits_per_sample":sample.bits_per_sample,"decoded_bytes":sample.samples.len(),
                    "decoded_sha256":digest(&sample.samples)})
                    });
                }
                let image = match kind {
                    "PICT" => graphics::decode_pict(&bytes),
                    "cicn" => graphics::decode_cicn(&bytes),
                    "ppat" => graphics::decode_pattern(&bytes),
                    "crsr" => graphics::decode_crsr(&bytes).map(|cursor| cursor.image),
                    _ => {
                        row["status"] = json!("no decoder adapter");
                        return Ok(Value::Null);
                    }
                }?;
                Ok(
                    json!({"kind":"graphics","width":image.width,"height":image.height,
                "decoded_bytes":image.rgba.len(),"decoded_sha256":digest(&image.rgba)}),
                )
            });
            match decoded {
                Ok(value) if !value.is_null() => {
                    row["status"] = json!("decoded");
                    row["decoded"] = value;
                }
                Ok(_) => {}
                Err(error) => {
                    row["status"] = json!("rejected");
                    row["error"] = json!(error.to_string());
                }
            }
            row
        })
        .collect();
    json!({"schema_version":1,"content_identity":content.identity,"resources":resources,
        "qualification":"Decoder results cover these input resources. Visual, playback and platform acceptance remain separate."})
}
