#include <string.h>
#include <stdio.h>
#include <pcap.h>
#include <stdlib.h>

#include <ctype.h>
#include <errno.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <net/ethernet.h>

#define IP_HL(ip)       (((ip)->ip_vhl) & 0x0f)
#define IP_V(ip)        (((ip)->ip_vhl) >> 4)

#define PARSE_CLIENT_PACKETS 1
#define PARSE_SERVER_PACKETS 2

struct eth_hdr {
        u_char  dhost[ETHER_ADDR_LEN];    /* destination host address */
        u_char  shost[ETHER_ADDR_LEN];    /* source host address */
        u_short type;                     /* IP? ARP? RARP? etc */
};

struct ip_hdr {
        u_char  ip_vhl;                 /* version << 4 | header length >> 2 */
        u_char  ip_tos;                 /* type of service */
        u_short ip_len;                 /* total length */
        u_short ip_id;                  /* identification */
        u_short ip_off;                 /* fragment offset field */
        u_char  ip_ttl;                 /* time to live */
        u_char  ip_p;                   /* protocol */
        u_short ip_sum;                 /* checksum */
        struct  in_addr ip_src,ip_dst;  /* source and dest address */
};

struct udp_hdr {
        u_short sport;               /* source port */
        u_short dport;               /* destination port */
        u_short len;
        u_short crc;
};

typedef struct name_parse {
    char *name;
    void (*parse)(u_char *data, u_char is_server);
} name_func;

void rx_sess (u_char *data, u_char is_server) {
    puts("sess");
}
void rx_rel (u_char *data, u_char is_server) {
    puts("rel");
}
void rx_ack (u_char *data, u_char is_server) {
    puts("ack");
}
void rx_beat (u_char *data, u_char is_server) {
    puts("");
}
void rx_mapreq (u_char *data, u_char is_server) {
    puts("");
}
void rx_mapdata (u_char *data, u_char is_server) {
    puts("");
}
void rx_objdata (u_char *data, u_char is_server) {
    puts("");
}
void rx_objack (u_char *data, u_char is_server) {
    puts("");
}
void rx_close (u_char *data, u_char is_server) {
    puts("");
}

name_parse msg_types[] = {
    [0] = { name =    "SESS", parse = rx_sess },
    [1] = { name =     "REL", parse = rx_rel },
    [2] = { name =     "ACK", parse = rx_ack },
    [3] = { name =    "BEAT", parse = rx_beat },
    [4] = { name =  "MAPREQ", parse = rx_mapreq },
    [5] = { name = "MAPDATA", parse = rx_mapdata },
    [6] = { name = "OBJDATA", parse = rx_objdata },
    [7] = { name =  "OBJACK", parse = rx_objack },
    [8] = { name =   "CLOSE", parse = rx_close }
}

void salem_parse (const u_char *data, u_char is_server) {
    printf((is_server)?"SERVER\n":"CLIENT\n");
    
}

void parse (u_char *user, const struct pcap_pkthdr *h, const u_char *bytes) {
    printf("%u %u%s\n", h->len, h->caplen, (h->len == h->caplen)?"":" !!! len != caplen");
    if (h->len != h->caplen) return;

    struct eth_hdr *eth = (struct eth_hdr*)(bytes);
    if (ntohs(eth->type) != ETHERTYPE_IP) {
        printf("not IP\n");
        return;
    }
    u_int i;
    for (i=0; i<6; ++i) { printf("%02x", eth->dhost[i]); }
    printf(" ");
    for (i=0; i<6; ++i) { printf("%02x", eth->shost[i]); }
    printf(" %04x\n", eth->type);

    struct ip_hdr *ip = (struct ip_hdr*)(bytes+sizeof(struct eth_hdr));
    int size_ip = IP_HL(ip)*4;
    if (size_ip != 20) {
        printf("wrong IP header length: %u\n", size_ip);
        return;
    }
    if (ip->ip_p != IPPROTO_UDP) {
        printf("not UDP\n");
        return;
    }
    printf("%s > ", inet_ntoa(ip->ip_src));
    printf("%s\n", inet_ntoa(ip->ip_dst));

    struct udp_hdr *udp = (struct udp_hdr*)(bytes + sizeof(struct eth_hdr) + sizeof(struct ip_hdr));
    printf("%u > %u\n", ntohs(udp->sport), ntohs(udp->dport));
    if (ntohs(udp->sport) == 1870) {
        if (ntohs(udp->dport) == 1870) {
            printf("sport == dport\n");
            return;
        }
        if (*user & PARSE_SERVER_PACKETS) {
            salem_parse(bytes + sizeof(struct eth_hdr) + sizeof(struct ip_hdr) + sizeof(struct udp_hdr), 1);
        }
    } else if (ntohs(udp->dport) == 1870) {
        if (*user & PARSE_CLIENT_PACKETS) {
            salem_parse(bytes + sizeof(struct eth_hdr) + sizeof(struct ip_hdr) + sizeof(struct udp_hdr), 0);
        }
    }

    printf("\n");
}

int main (int argc, char *argv[]) {
    u_char to_parse;
    if (argc != 3) {
        puts("wrong arguments count");
        exit(1);
    }
    if (strcmp(argv[2],"client") == 0) {
        to_parse = 1;
    } else if (strcmp(argv[2],"server") == 0) {
        to_parse = 2;
    } else if (strcmp(argv[2],"both") == 0) {
        to_parse = 3;
    } else {
        printf("wrong parse type '%s'\n", argv[2]);
        exit(1);
    }
    char errbuf[PCAP_ERRBUF_SIZE];
    pcap_t *pcap = pcap_open_offline(argv[1], errbuf);
    if (pcap == NULL) {
        printf("error '%s' while opening pcap file '%s'\n", errbuf, argv[1]);
        exit(1);
    }
    pcap_dispatch(pcap, -1, parse, &to_parse);
    pcap_close(pcap);
    exit(0);
}
