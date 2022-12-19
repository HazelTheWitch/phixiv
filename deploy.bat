@echo off

docker run --rm -v %cd%:/code -v %userprofile%/.cargo/registry:/root/.cargo/registry -v %userprofile%/.cargo/git:/root/.cargo/git rustserverless/lambda-rust

aws lambda update-function-code --function-name phixiv --zip-file "fileb://./target/lambda/release/phixiv.zip"
aws lambda update-function-code --function-name phixiv_proxy --zip-file "fileb://./target/lambda/release/phixiv_proxy.zip"