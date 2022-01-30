import requests

shares = ["AdA5O7dPevK6g3towGKnTAT+R+t0QDBLaDaPg9Mr8Zry", "AkmjLRSRaPFPU43BUiTKEwNHa3PV5C7xphwGNGQYkoUa", "A3hhvnd7dLHKFQNF3gDzeO841kyvbv1KGA6mWAwBT1TR", "BMqgqaaUM6fVlvB+/oJSSrNIIHi5vJOjktPGpgiTFeyW", "BfnA+lNUBd28WKHFVK93KHx5iDPlvZK/IC8CsXbJCmfD", "Bkt/FS2RLdZqwb+hkQoaTQCiZx8jh6Tb4puQrnvSGH1o", "BwN59ZB3ZOnydRfdF4CzDer0UWgyt5ed0irJeoU61eRe", "CPS73LRskpjXkVVrzxO8GB5s+9TWZfwQFJbMEoDnA/hR"]

for share in shares:
	response = requests.post("http://127.0.0.1:59090/v1/shares", headers = {"content-type": "application/json"}, json={"value": share})
	print(response)


response = requests.post("http://127.0.0.1:59090/v1/secrets/unseal")
print(response)
