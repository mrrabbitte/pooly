import requests

shares = [
"AaKptGKEtloj8zxtKMfmor0Mm92DEZmDFGHx2fiZrT17",
"Aj7dMfl03FP3qC/51Ya/seJEaYXYIIrZUuhRzExWR/Jp",
"A5OCZnSl3dYVY4VLK4LQ/jLCmMfwFJkyKeoYM6fWOCDh",
"BGsfKEOKHbdf6JowhRoVsGDwcH+T1x4TnLdgXu9nZK17",
"BfEw5aF28lfR9QjWxvVB60N5RTa9jmTJ0uAiPFGlLfeh",
"Bp0G6ALVNaQwNwHljFv//mfohOIPpxQh1OrVwyKbd++X",
"B5iPHzJys5oKanh9nAsDUF2z44Uf5YUToAb8YA2jK9Ub",
"CJKty90MV9UxBZRFps8vCGX7SXlUJ5EmTfKYVKnZo6l3"
]

for share in shares:
	response = requests.post("http://127.0.0.1:59090/v1/shares", headers = {"content-type": "application/json"}, json={"value": share})
	print(response)


response = requests.post("http://127.0.0.1:59090/v1/secrets/unseal")
print(response)
