use anyhow::Result;
use tiktoken_rs::p50k_base;

pub(crate) fn tokenize(text: &str) -> Result<Vec<f32>> {
    let tokens = tokenize_raw(text)?;
    Ok(convert_format(tokens))
}

fn tokenize_raw(text: &str) -> Result<Vec<usize>> {
    let bpe = p50k_base()?;

    Ok(bpe.encode_with_special_tokens(text))
}

fn convert_format(tokens: Vec<usize>) -> Vec<f32> {
    tokens.iter().map(|&x| x as f32).collect()
}
