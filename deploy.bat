@echo off

cargo lambda build --release --arm64 --output-format zip

aws lambda update-function-code --function-name phixiv --architectures "arm64" --zip-file "fileb://./target/lambda/phixiv/bootstrap.zip"
aws lambda update-function-code --function-name phixiv_proxy --architectures "arm64" --zip-file "fileb://./target/lambda/phixiv_proxy/bootstrap.zip"