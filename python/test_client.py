import payloads_pb2 as p
import requests

qr = p.QueryRequest()

qr.db_id = "pooly_test"
qr.query = "select column1, column2 from newtable where column1 = $1;"

vw = p.ValueWrapper()
vw.string = "something"

qr.params.append(vw)

print("Sending: \n", qr)

response = requests.post("http://127.0.0.1:59090/query", headers={"content-type": "application/protobuf"}, data=qr.SerializeToString())

rec = p.QueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print("\n Received: \n")
print(rec)
