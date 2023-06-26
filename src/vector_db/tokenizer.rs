use anyhow::Result;
use tiktoken_rs::p50k_base;

pub(crate) fn tokenize(text: &str) -> Result<Vec<f32>> {
    let tokens = tokenize_raw(text)?;
    Ok(convert_from_usize_to_f32(tokens))
}

pub(crate) fn detokenize(tokens: Vec<f32>) -> Result<String> {
    let text = detokenize_raw(convert_from_f32_to_usize(tokens))?;
    Ok(text)
}

fn tokenize_raw(text: &str) -> Result<Vec<usize>> {
    let bpe = p50k_base()?;

    Ok(bpe.encode_with_special_tokens(text))
}

fn detokenize_raw(tokens: Vec<usize>) -> Result<String> {
    let bpe = p50k_base()?;
    let text = bpe.decode(tokens)?;
    Ok(text)
}

fn convert_from_usize_to_f32(tokens: Vec<usize>) -> Vec<f32> {
    tokens.iter().map(|&x| x as f32).collect()
}

fn convert_from_f32_to_usize(tokens: Vec<f32>) -> Vec<usize> {
    tokens.iter().map(|&x| x as usize).collect()
}
