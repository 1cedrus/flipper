echo "🧱 Building contract..."
cargo contract build 

echo "🔥 Uploading contract..."
cargo contract upload --suri //Alice -x ./target/ink/flipper.contract

echo "✅ Done!"
