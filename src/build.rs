fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("BUILD start);
    tonic_build::compile_protos("proto/helloworld.proto")?;
    println!("BUILD end");
    Ok(())
}
