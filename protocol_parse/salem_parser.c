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

typedef struct {
        u_char  dhost[ETHER_ADDR_LEN];    /* destination host address */
        u_char  shost[ETHER_ADDR_LEN];    /* source host address */
        u_short type;                     /* IP? ARP? RARP? etc */
} eth_hdr;

typedef struct {
        u_char  ip_vhl;                 /* version << 4 | header length >> 2 */
        u_char  ip_tos;                 /* type of service */
        u_short ip_len;                 /* total length */
        u_short ip_id;                  /* identification */
        u_short ip_off;                 /* fragment offset field */
        u_char  ip_ttl;                 /* time to live */
        u_char  ip_p;                   /* protocol */
        u_short ip_sum;                 /* checksum */
        struct  in_addr ip_src,ip_dst;  /* source and dest address */
} ip_hdr;

typedef struct {
        u_short sport;               /* source port */
        u_short dport;               /* destination port */
        u_short len;
        u_short crc;
} udp_hdr;

/*
typedef struct {
    u_char type;
    union {
        struct {
            u16  unknown;
            zstr proto;
            u16  ver;
            zstr user;
            u8[] cookie;
        } sess_client;
        struct {
            u8 error;
        } sess_server;
        struct {
            u16 seq;
            struct {
                u8 type;
                ...
            } rels[];
        } rel;
        struct {
        } ack;
        struct {
        } beat;
        struct {
        } mapreq;
        struct {
        } mapdata;
        struct {
        } objdata;
        struct {
        } objack;
        struct {
        } close;
    };
} salem_message;
*/

/* PROOF OF CONCEPT */
typedef struct {
    u_short *unknown;
    char    *proto;
    u_short *ver;
    char    *user;
    u_char  *cookie;
} client_sess;
typedef struct {
    u_char *error;
} server_sess;
typedef struct {
    u_char *type;
    // ...
} rel_element;
typedef struct {
    u_short *seq;
    rel_element *rels;
    u_int rels_len;
} rel;
typedef struct {
} ack;
typedef struct {
} beat;
typedef struct {
} mapreq;
typedef struct {
} mapdata;
typedef struct {
} objdata;
typedef struct {
} objack;
typedef struct {
} close;
typedef struct {
    u_char *type;
    union {
        client_sess c_sess;
        server_sess s_sess;
        rel rel;
        ack ack;
        beat beat;
        mapreq mapreq;
        mapdata mapdata;
        objdata objdata;
        objack objack;
        close close;
    };
} salem_message;

typedef struct {
    u_char *data;
    u_short len;
    u_char from_server;
} message;

u_char *u8 (message *msg) {
    if (msg->len < 1) abort();
    u_char *ret = msg->data;
    ++msg->data;
    --msg->len;
    return ret;
}

u_short *u16 (message *msg) {
    if (msg->len < 2) abort();
    u_short *ret = (u_short *)msg->data;
    msg->data += 2;
    msg->len -= 2;
    return ret;
}

char *zstr (message *msg) {
    if (msg->len < 1) abort();
    char *ret = msg->data;
    while (1) {
        if (msg->data[0] == 0) {
            ++msg->data;
            --msg->len;
            return ret;
        }
        ++msg->data;
        --msg->len;
        if (msg->len == 0) abort();
    }
}

rel_element *new_rel_element(rel *rel) {
    ++rel->rels_len;
    rel->rels = realloc(rel->rels, sizeof(rel_element) * rel->rels_len);
    rel_element *last_rel = &rel->rels[rel->rels_len-1];
    memset(last_rel, 0, sizeof(rel_element));
    return last_rel;
}

void map_to_rel_element (message *msg, rel_element *el) {
    el->type = u8(msg);
}

void map_to_rel (message *msg, salem_message *smsg) {
    smsg->rel.seq = u16(msg);
    while (msg->len > 0) {
        rel_element *rel_el = new_rel_element(&smsg->rel);
        map_to_rel_element(msg, rel_el);
    }
}

void map_to_client_sess (message *msg, salem_message *smsg) {
    smsg->c_sess.unknown = u16(msg);
    smsg->c_sess.proto = zstr(msg);
    smsg->c_sess.ver = u16(msg);
    smsg->c_sess.user = zstr(msg);
}

void map_to_server_sess (message *msg, salem_message *smsg) {
    smsg->s_sess.error = u8(msg);
}

void map_to_sess (message *msg, salem_message *smsg) {
    if (msg->from_server) map_to_server_sess(msg, smsg);
    else map_to_client_sess(msg, smsg);
}

void map_to_ack (message *msg, salem_message *smsg) {}
void map_to_beat (message *msg, salem_message *smsg) {}
void map_to_mapreq (message *msg, salem_message *smsg) {}
void map_to_mapdata (message *msg, salem_message *smsg) {}
void map_to_objdata (message *msg, salem_message *smsg) {}
void map_to_objack (message *msg, salem_message *smsg) {}
void map_to_close (message *msg, salem_message *smsg) {}
/******************/

/*
u_char u8 (message *msg) {
    if (msg->len < 1) {
        printf("!!! u8 FAILED !!!");
        abort();
    }
    u_char ret = msg->data[0];
    msg->data += 1;
    msg->len -= 1;
    return ret;
}

u_short u16 (message *msg) {
    u_short ret = u8(msg);
    ret += ((u_short)u8(msg)) << 8;
    return ret;
}

char *cstr (message *msg) {
    char *str = NULL;
    int i = -1;
    do {
        ++i;
        str = realloc(str, i + 1);
        str[i] = u8(msg);
    } while (str[i] != 0);
    return str;
}

u_char *bytes (message *msg, u_short len) {
    u_char *b;
    if (len > 0) {
        if (len > msg->len) {
            abort();
        }
        b = malloc(len);
        memcpy(b, msg->data, len);
        msg->data += len;
        msg->len -= len;
    } else {
        b = malloc(msg->len);
        memcpy(b, msg->data, msg->len);
        msg->len = 0;
    }
    return b;
}
*/

char *sess_errors[] = {
    [0] = "OK",
    [1] = "AUTH",
    [2] = "BUSY",
    [3] = "CONN",
    [4] = "PVER",
    [5] = "EXPR"
};

/*
void rx_sess (message *msg) {
    if (msg->from_server) {
        u_char error = u8(msg);
        printf("    error=%u (%s)\n", error, sess_errors[error]);
    } else {
        u_short unknown = u16(msg);
        char *proto = cstr(msg);
        u_short ver = u16(msg);
        char *user = cstr(msg);
        printf("    unknown=%hu proto='%s' ver=%hu user='%s' cookie=TODO\n", unknown, proto, ver, user);
        free(proto);
        free(user);
    }
}

void rx_rel (message *msg) {
    u_char seq = u16(msg);
    printf("    seq=%u\n", seq);
    while (msg->len > 0) {
        u_char type = u8(msg);
        u_short len;
        if ((type & 0x80) != 0) {
            type &= 0x7f;
            len = u16(msg);
        } else {
            len = msg->len;
        }
        printf("      type=%u len=%hu\n", type, len);
        u_char *rel = bytes(msg, len);
        free(rel);
    }
}

void rx_ack (message *msg) {
    u_short seq = u16(msg);
    printf("    seq=%hu\n", seq);
}
void rx_beat (message *msg) {
}
void rx_mapreq (message *msg) {
}
void rx_mapdata (message *msg) {
}
void rx_objdata (message *msg) {
}
void rx_objack (message *msg) {
}
void rx_close (message *msg) {
}
*/

typedef struct {
    char *name;
    void (*map)(message *msg, salem_message *smsg);
} name_map_print;

name_map_print msg_types[] = {
    [0] = { .name =    "SESS", .map = map_to_sess,  /*, .print = print_sess   */ },
    [1] = { .name =     "REL", .map = map_to_rel    /*, .print = print_rel    */ },
    [2] = { .name =     "ACK", .map = map_to_ack    /*, .print = print_ack    */ },
    [3] = { .name =    "BEAT", .map = map_to_beat   /*, .print = print_beat   */ },
    [4] = { .name =  "MAPREQ", .map = map_to_mapreq /*, .print = print_mapreq */ },
    [5] = { .name = "MAPDATA", .map = map_to_mapdata/*, .print = print_mapdata*/ },
    [6] = { .name = "OBJDATA", .map = map_to_objdata/*, .print = print_objdata*/ },
    [7] = { .name =  "OBJACK", .map = map_to_objack /*, .print = print_objack */ },
    [8] = { .name =   "CLOSE", .map = map_to_close  /*, .print = print_close  */ }
};

void map_to_any (message *msg, salem_message *smsg) {
    smsg->type = u8(msg);
    //printf("  %s\n", msg_types[type].name);
    msg_types[*smsg->type].map(msg, smsg);
}

void salem_parse (message *msg) {
    printf((msg->from_server)?"SERVER\n":"CLIENT\n");
    salem_message smsg;
    memset(&smsg, 0, sizeof(salem_message));
    map_to_any(msg, &smsg);
    if (msg->len > 0) {
        printf("DATA REMAINS %u bytes\n", msg->len);
    }
}

void parse (u_char *user, const struct pcap_pkthdr *h, const u_char *bytes) {
    //printf("%u %u%s\n", h->len, h->caplen, (h->len == h->caplen)?"":" !!! len != caplen");
    if (h->len != h->caplen) {
        printf("len != caplen");
        return;
    }
    if (h->len <= sizeof(eth_hdr) + sizeof(ip_hdr) + sizeof(udp_hdr)) {
        puts("too small frame");
        return;
    }

    eth_hdr *eth = (eth_hdr*)(bytes);
    if (ntohs(eth->type) != ETHERTYPE_IP) {
        printf("not IP\n");
        return;
    }
    //u_int i;
    //for (i=0; i<6; ++i) { printf("%02x", eth->dhost[i]); }
    //printf(" ");
    //for (i=0; i<6; ++i) { printf("%02x", eth->shost[i]); }
    //printf(" %04x\n", eth->type);

    ip_hdr *ip = (ip_hdr *)(bytes + sizeof(eth_hdr));
    int size_ip = IP_HL(ip)*4;
    if (size_ip != 20) {
        printf("wrong IP header length: %u\n", size_ip);
        return;
    }
    if (ip->ip_p != IPPROTO_UDP) {
        printf("not UDP\n");
        return;
    }
    //printf("%s > ", inet_ntoa(ip->ip_src));
    //printf("%s\n", inet_ntoa(ip->ip_dst));

    udp_hdr *udp = (udp_hdr*)(bytes + sizeof(eth_hdr) + sizeof(ip_hdr));
    //printf("%u > %u\n", ntohs(udp->sport), ntohs(udp->dport));

    message msg;
    msg.data = (u_char *)bytes + sizeof(eth_hdr) + sizeof(ip_hdr) + sizeof(udp_hdr);
    msg.len = h->len - sizeof(eth_hdr) - sizeof(ip_hdr) - sizeof(udp_hdr);
    if (ntohs(udp->sport) == 1870) {
        if (udp->sport == udp->dport) {
            printf("sport == dport\n");
            return;
        }
        if (*user & PARSE_SERVER_PACKETS) {
            msg.from_server = 1;
            salem_parse(&msg);
        }
    } else if (ntohs(udp->dport) == 1870) {
        if (*user & PARSE_CLIENT_PACKETS) {
            msg.from_server = 0;
            salem_parse(&msg);
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
