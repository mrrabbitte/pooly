import payloads_pb2 as p
import requests

config = {
  "hosts": ["localhost"],
  "db_name": "pooly_test",
  "user": "pooly",
  "password": "pooly_pooly_123",
  "max_connections": 10
}

response = requests.post("http://127.0.0.1:59090/v1/configs", headers = {"content-type": "application/json"}, json=config)

print(response)

qr = p.QueryRequest()

qr.db_id = "pooly_test"
qr.query = "select column1, column2 from newtable where column1 = $1 and column2 = $2;"

vw1 = p.ValueWrapper()
vw1.string = "something"

vw2 = p.ValueWrapper()
vw2.int8 = 1

qr.params.append(vw1)
qr.params.append(vw2)

print("Sending: \n", qr)

response = requests.post("http://127.0.0.1:59090/v1/query", headers={"content-type": "application/protobuf"}, data=qr.SerializeToString())

rec = p.QueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print("\n Received: \n")
print(rec)
