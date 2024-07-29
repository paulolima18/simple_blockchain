# simple_blockchain

curl -X POST http://127.0.0.1:3030/mine/1
curl -X POST http://127.0.0.1:3030/transaction -H "Content-Type: application/json" -d '{"sender":"Alice","receiver":"Bob","amount":50}'