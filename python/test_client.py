import payloads_pb2 as p
import requests

BASE_URL = "http://127.0.0.1:8868"
CONFIGS = BASE_URL + "/a/v1/configs"
QUERY = BASE_URL + "/c/v1/query"
BULK = BASE_URL + "/c/v1/bulk"

config = {
  "hosts": ["localhost"],
  "db_name": "pooly_test",
  "user": "pooly",
  "password": "pooly_pooly_123",
  "max_connections": 10
}

response = requests.post(CONFIGS, headers = {"content-type": "application/json"}, json=config)

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

response = requests.post(QUERY, headers={"content-type": "application/protobuf"}, data=qr.SerializeToString())

rec = p.QueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print("\n Received: \n")
print(rec)


qr = p.QueryRequest()

qr.db_id = "pooly_test"
qr.query = "insert into newtable(column1, column2) values($1, $2) returning *;"
vw1 = p.ValueWrapper()
vw1.string = "something"

vw2 = p.ValueWrapper()
vw2.int8 = 1
qr.params.append(vw1)
qr.params.append(vw2)
print("Sending: \n", qr)

response = requests.post(QUERY, headers={"content-type": "application/protobuf"}, data=qr.SerializeToString())

rec = p.QueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print("\n Received: \n")
print(rec)

bulk = p.TxBulkQueryRequest()
bulk.db_id = "pooly_test"
for i in range(0, 1):
	body = p.TxBulkQueryRequestBody()
	body.query = "insert into newtable(column1, column2) values ($1, $2) returning *;"
	for j in range(0, 10):
		params_row = p.TxBulkQueryParams()
		vw1 = p.ValueWrapper()
		vw1.string = "tx_bulk_test_{0}".format(j)
		vw2 = p.ValueWrapper()
		vw2.int8 = j
		params_row.values.append(vw1)
		params_row.values.append(vw2)
		body.params.append(params_row) 
	bulk.queries.append(body)

response = requests.post(BULK, headers={"content-type": "application/protobuf"}, data=bulk.SerializeToString())

rec = p.TxBulkQueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print(response.headers)
print("\n Received: \n")
print(rec)
