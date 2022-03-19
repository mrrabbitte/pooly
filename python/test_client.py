import payloads_pb2 as p
import requests
import base64 
import jwt

### Please note, this is a dirty script written on a knee, a proper Python client will come. 

BASE_URL = "http://127.0.0.1:8868"

INITIALIZE = BASE_URL + "/i/v1/secrets/initialize"
CREATE_KEY = BASE_URL + "/a/v1/keys"
CONFIGS = BASE_URL + "/a/v1/connections"
QUERY = BASE_URL + "/c/v1/query"
BULK = BASE_URL + "/c/v1/bulk"
CONTENT_TYPE = {"content-type": "application/json"}

ACCESS_CONNECTION_IDS = "/v1/access/literals"

CLIENT_ID = "client-id-1"
CONNECTION_ID = "connection-id-1"

JWT_KEY = b"ZDR1S0E0WEUwY0lmWnBweXUwYmFiM2s5aGlWUUxTZ2VUcldrcTV1ZGZnZGY="

INIT_AUTH = "uUBlr1SyHb3ETk5h2A6yNrjXRa99FhopQ6Ow53XtXxrXC4IoTVT0o2fbXKDyBHS19scDFtl5aZlTRk"

key = {"kid":"kid-1", "alg":"Hs512", "value": base64.b64encode(JWT_KEY)}

response = requests.post(INITIALIZE, headers = {"content-type": "application/json", "Authorization": INIT_AUTH}, json=key)

print(response)
print(response.content)


admin_token = jwt.encode({"exp": 1700000000, "aud": "pooly", "pooly_role": "admin", "sub": "admin-id-1"}, JWT_KEY, algorithm="HS512",  headers = {"kid": "kid-1", "typ": "JWT", "cty": "JWT"})

print(admin_token)

input("wat now?")

config = {
  "id": CONNECTION_ID,
  "hosts": ["localhost"],
  "db_name": "pooly_test",
  "user": "pooly",
  "password": "pooly_pooly_123",
  "ports": [5432],
  "max_connections": 10
}

response = requests.post(CONFIGS, headers = {"content-type": "application/json", "Authorization": "Bearer " + admin_token}, json=config)

print(response)
print(response.content)

client_token = jwt.encode({"exp": 1700000000, "aud": "pooly", "pooly_role": "client_service", "sub": CLIENT_ID}, JWT_KEY, algorithm="HS512",  headers = {"kid": "kid-1", "typ": "JWT", "cty": "JWT"})


qr = p.QueryRequest()

qr.connection_id = CONNECTION_ID
qr.query = "select column1, column2 from newtable where column1 = $1 and column2 = $2;"

vw1 = p.ValueWrapper()
vw1.string = "something"

vw2 = p.ValueWrapper()
vw2.int8 = 1

qr.params.append(vw1)
qr.params.append(vw2)

print("Sending: \n", qr)

response = requests.post(QUERY, headers={"content-type": "application/protobuf", "Authorization": "Bearer " + client_token}, data=qr.SerializeToString())

print("\n Got:", response.content, response)
print("\n Received: \n")

rec = p.QueryResponse()
rec.ParseFromString(response.content)

print(rec)


qr = p.QueryRequest()

qr.connection_id = CONNECTION_ID
qr.query = "insert into newtable(column1, column2) values($1, $2) returning *;"
vw1 = p.ValueWrapper()
vw1.string = "something"

vw2 = p.ValueWrapper()
vw2.int8 = 1
qr.params.append(vw1)
qr.params.append(vw2)
print("Sending: \n", qr)

response = requests.post(QUERY, headers={"content-type": "application/protobuf", "Authorization": "Bearer " + client_token}, data=qr.SerializeToString())

rec = p.QueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print("\n Received: \n")
print(rec)

bulk = p.TxBulkQueryRequest()
bulk.connection_id = CONNECTION_ID
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

response = requests.post(BULK, headers={"content-type": "application/protobuf", "Authorization": "Bearer " + client_token}, data=bulk.SerializeToString())

rec = p.TxBulkQueryResponse()
rec.ParseFromString(response.content)

print("\n Got:", response)
print(response.headers)
print("\n Received: \n")
print(rec)
