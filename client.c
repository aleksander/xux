#include <stdio.h>
#include <openssl/ssl.h>
#include <openssl/err.h>
#include <openssl/bio.h>

int main( void ) {
	SSL_library_init();
	SSL_load_error_strings();
	SSL_CTX *ctx = SSL_CTX_new(TLSv1_client_method());
	if(ctx) {
		SSL *ssl = SSL_new( ctx );
		if(ssl) {
			BIO *bio = BIO_new_connect("moltke.seatribe.se:1871");
			if(bio) {
				SSL_set_bio(ssl, bio, bio);
				if(SSL_connect(ssl) > 0) {
					char buf[] = {1,7,'s','a','l','v','i','a','n'};
					if(SSL_write(ssl, (void *)buf, sizeof(buf)) > 0) {
						char r[100];
						if(SSL_read(ssl, (void *)r, 100) > 0) {
							printf("readed: ");
							int i;
							for(i=0; i<100; ++i) {
								printf("%02d", r[i]);
							}
							printf("\n");
						} else {
							printf("read(): %s\n", ERR_error_string(ERR_get_error(), NULL));
						}

					} else {
						printf("write(): %s\n", ERR_error_string(ERR_get_error(), NULL));
					}

					while(1) {
						int ret = SSL_shutdown(ssl);
						if(ret == -1) {
							printf("SSL_shutdown()->%d: %s\n", ret, ERR_error_string(ERR_get_error(), NULL));
							break;
						}
						if(ret == 1) {
							break;
						}
					}
				} else {
					printf("SSL_connect(): %s\n", ERR_error_string(ERR_get_error(), NULL));
				}
			}
			SSL_free(ssl);
		}
		SSL_CTX_free(ctx);
	}
	ERR_free_strings();
	exit(EXIT_SUCCESS);
}
