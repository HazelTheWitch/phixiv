@echo off

cargo lambda build --release --arm64 --output-format zip

aws lambda update-function-code --function-name phixiv --architectures "arm64" --zip-file "fileb://./target/lambda/phixiv_main/bootstrap.zip"
aws lambda update-function-code --function-name phixiv_proxy --architectures "arm64" --zip-file "fileb://./target/lambda/phixiv_proxy/bootstrap.zip"
aws lambda update-function-code --function-name ppxiv_redirect --architectures "arm64" --zip-file "fileb://./target/lambda/ppxiv_redirect/bootstrap.zip"
aws lambda update-function-code --function-name phixiv_oembed --architectures "arm64" --zip-file "fileb://./target/lambda/ppxiv_redirect/bootstrap.zip"