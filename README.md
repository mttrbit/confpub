# confpub (confluence publisher)
A cli tool for publishing Confluence pages to Confluence instance protected by saml + vip.

# Usage

``` sh
# display help
confpub --help 


# upload pages using plain text credentials
confpub -u username:password -e https://your.endpoint.com path/to/pages.yml

# upload pages using encrypted credentials
confpub -k demo-private-key.pem -f user_encrypted.txt -e https://your.endpoint.com path/to/pages.yml
```


# How to use private keys

``` sh

# generate a private key
openssl genrsa -out demo-private-key.pem 2048

# generate a public key
openssl rsa -in demo-private-key.pem -pubout -out demo-public-key.pem

# create a file user.txt with content your_email:your password

# encrypt file user.txt
openssl rsautl -in user.txt -out user_encrypted.txt -inkey demo-public-key.pem -pubin encrypt
```
