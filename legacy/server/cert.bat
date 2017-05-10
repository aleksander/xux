openssl genrsa -out key.pem
openssl req -new -key key.pem -out csr.pem
openssl x509 -req -days 3650 -in csr.pem -signkey key.pem -out crt.pem