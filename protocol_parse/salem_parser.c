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

#include <assert.h>

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





//////////////////////////////////////////////////////////////////////////////////////





typedef struct {
    u_char *bytes;
    size_t len;
} bseq;

typedef struct {
    int32_t *x;
    int32_t *y;
} coord_t;

typedef struct {
    u_short *unknown;
    char    *proto;
    u_short *ver;
    char    *user;
    bseq     cookie;
} client_sess;

typedef struct {
    u_char *err;
    bseq    trash;
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
    u_short *seq;
} ack;
typedef struct {
} beat;
typedef struct {
} mapreq;
typedef struct {
} mapdata;
typedef struct {
} objack;

typedef struct {
    //TODO substructure all this
    u_char   *fl;
    int32_t  *id;
    int32_t  *frame;
    coord_t   move_coord;
    coord_t   linbeg_s;
    coord_t   linbeg_t;
    int32_t  *linbeg_c;
    int16_t  *speech_zo;
    char     *speech_text;
    uint16_t *move_ia;
    int32_t  *linstep;
    uint16_t *res_id;
    bseq      res_sdt;
    uint8_t  *health;
    uint16_t *compose_resid;
    coord_t   draw_off;
    coord_t   lumin_off;
    uint16_t *lumin_sz;
    uint8_t  *lumin_str;
    uint32_t *follow_oid;
    uint16_t *follow_xfres;
    char     *follow_xfname;
    uint32_t *homing_oid;
    coord_t   homing_coord;
    uint16_t *homing_v;
    int32_t  *overlay_id;
    uint16_t *overlay_resid;
    bseq      overlay_sdt;
    char     *buddy_name;
    uint8_t  *buddy_group;
    uint8_t  *buddy_type;
    uint16_t *icon_resid;
    uint8_t  *icon_ifl;
} objdata_element;

typedef struct {
    objdata_element *objs;
    u_int objs_len;
} objdata;

typedef struct {
} close;
typedef struct {
    u_char *type;
    u_char from_server;
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
    assert(msg->len >= 1);
    u_char *ret = msg->data;
    ++msg->data;
    --msg->len;
    return ret;
}

u_short *u16 (message *msg) {
    assert(msg->len >= 2);
    u_short *ret = (u_short *)msg->data;
    msg->data += 2;
    msg->len -= 2;
    return ret;
}

int16_t *s16 (message *msg) {
    assert(msg->len >= 2);
    int16_t *ret = (int16_t *)msg->data;
    msg->data += 2;
    msg->len -= 2;
    return ret;
}

uint32_t *u32 (message *msg) {
    assert(msg->len >= 4);
    uint32_t *ret = (uint32_t *)msg->data;
    msg->data += 4;
    msg->len -= 4;
    return ret;
}

int32_t *s32 (message *msg) {
    assert(msg->len >= 4);
    int32_t *ret = (int32_t *)msg->data;
    msg->data += 4;
    msg->len -= 4;
    return ret;
}

char *zstr (message *msg) {
    assert(msg->len >= 1);
    char *ret = (char *)msg->data;
    while (1) {
        if (msg->data[0] == 0) {
            ++msg->data;
            --msg->len;
            return ret;
        }
        ++msg->data;
        --msg->len;
        assert(msg->len != 0);
    }
}

coord_t coord (message *msg) {
    coord_t c;
    c.x = s32(msg);
    c.y = s32(msg);
    return c;
}

bseq bytes (message *msg, u_int num) {
    bseq ret;
    assert(msg->len >= 1);
    assert(num == 0 || msg->len >= num);
    ret.bytes = msg->data;
    ret.len = (num > 0)?num:msg->len;
    msg->data += (num > 0)?num:msg->len;
    msg->len -= (num > 0)?num:msg->len;
    return ret;
}

char *print_bseq (bseq *seq) {
    static char ret[512];
    if ((seq->len * 3 - 1) > 511) {
        sprintf(ret, "too long bytes sequence");
        return ret;
    }
    u_int i;
    u_int pos = 0;
    memset(ret, 0, 512);
    for (i=0; i<seq->len; ++i) {
        sprintf(ret+pos, "%s%02X", (i)?" ":"", seq->bytes[i]);
        pos += (i)?3:2;
    }
    return ret;
}

char *print_bseq_text (bseq *seq) {
    static char ret[512];
    if ((seq->len * 2) > 511) {
        sprintf(ret, "too long bytes sequence");
        return ret;
    }
    u_int i;
    memset(ret, 0, 512);
    for (i=0; i<seq->len; ++i) {
        char c = seq->bytes[i];
        sprintf(ret+i, "%c", (isprint(c))?c:'.');
    }
    return ret;
}

//////  REL  /////////////////////////////////////////////////////////////////////////

rel_element *new_rel_element(rel *rel) {
    ++rel->rels_len;
    rel->rels = realloc(rel->rels, sizeof(rel_element) * rel->rels_len);
    rel_element *last_rel = &rel->rels[rel->rels_len-1];
    memset(last_rel, 0, sizeof(rel_element));
    return last_rel;
}

void map_to_rel_element (message *msg, rel_element *el) {
    el->type = u8(msg);
    u_short len;
    if ((*el->type & 0x80) != 0) {
        *el->type &= 0x7f;
        len = *u16(msg);
    } else {
        len = msg->len;
    }
    bseq rel = bytes(msg, len);
}

void map_to_rel (message *msg, salem_message *smsg) {
    smsg->rel.seq = u16(msg);
    while (msg->len > 0) {
        rel_element *el = new_rel_element(&smsg->rel);
        map_to_rel_element(msg, el);
    }
}

//////  SESS  ////////////////////////////////////////////////////////////////////////

char *sess_errors[] = {
    [0] = "OK",
    [1] = "AUTH",
    [2] = "BUSY",
    [3] = "CONN",
    [4] = "PVER",
    [5] = "EXPR"
};

void map_to_client_sess (message *msg, salem_message *smsg) {
    smsg->c_sess.unknown = u16(msg);
    smsg->c_sess.proto = zstr(msg);
    smsg->c_sess.ver = u16(msg);
    smsg->c_sess.user = zstr(msg);
    smsg->c_sess.cookie = bytes(msg, 0);
}

void map_to_server_sess (message *msg, salem_message *smsg) {
    smsg->s_sess.err = u8(msg);
    //smsg->s_sess.trash = bytes(msg, 0);
}

void map_to_sess (message *msg, salem_message *smsg) {
    if (msg->from_server) map_to_server_sess(msg, smsg);
    else map_to_client_sess(msg, smsg);
}

void print_sess (salem_message *smsg) {
    if (smsg->from_server) printf("    err=%u %s trash=[%s]\n",
       *smsg->s_sess.err, sess_errors[*smsg->s_sess.err], print_bseq_text(&smsg->s_sess.trash));
    else printf("    unknown=%hu proto=%s ver=%hu user=%s cookie=[%s]\n",
       *smsg->c_sess.unknown, smsg->c_sess.proto, *smsg->c_sess.ver, smsg->c_sess.user, print_bseq_text(&smsg->c_sess.cookie));
}

//////  ACK  /////////////////////////////////////////////////////////////////////////

void map_to_ack (message *msg, salem_message *smsg) {
    smsg->ack.seq = u16(msg);
}

//////  BEAT  ////////////////////////////////////////////////////////////////////////

void map_to_beat (message *msg, salem_message *smsg) {}

//////  MAPREQ  //////////////////////////////////////////////////////////////////////

void map_to_mapreq (message *msg, salem_message *smsg) {}

//////  MAPDATA  /////////////////////////////////////////////////////////////////////

void map_to_mapdata (message *msg, salem_message *smsg) {}

//////  OBJDATA  /////////////////////////////////////////////////////////////////////

objdata_element *new_objdata_element (objdata *obj) {
    ++obj->objs_len;
    obj->objs = realloc(obj->objs, sizeof(objdata_element) * obj->objs_len);
    objdata_element *last_obj = &obj->objs[obj->objs_len-1];
    memset(last_obj, 0, sizeof(objdata_element));
    return last_obj;
}

typedef struct {
    char *name;
    void (*parse)(message *msg, objdata_element *el);
} name_parse;

void rx_objdata_rem (message *msg, objdata_element *el) {
}

void rx_objdata_move (message *msg, objdata_element *el) {
    el->move_coord = coord(msg);
    el->move_ia = u16(msg);
}

void rx_objdata_res (message *msg, objdata_element *el) {
    el->res_id = u16(msg);
    if ((*el->res_id & 0x8000) != 0) {
        el->res_sdt = bytes(msg, *u8(msg));
    }
}

void rx_objdata_linbeg (message *msg, objdata_element *el) {
    el->linbeg_s = coord(msg);
    el->linbeg_t = coord(msg);
    el->linbeg_c = s32(msg);
}

void rx_objdata_linstep (message *msg, objdata_element *el) {
    el->linstep = s32(msg);
}

void rx_objdata_speech (message *msg, objdata_element *el) {
    el->speech_zo = s16(msg);
    el->speech_text = zstr(msg);
}

void rx_objdata_compose (message *msg, objdata_element *el) {
    el->compose_resid = u16(msg);
}

void rx_objdata_drawoff (message *msg, objdata_element *el) {
    el->draw_off = coord(msg);
}

void rx_objdata_lumin (message *msg, objdata_element *el) {
    el->lumin_off = coord(msg);
    el->lumin_sz = u16(msg);
    el->lumin_str = u8(msg);
}

void rx_objdata_avatar (message *msg, objdata_element *el) {
    //TODO
    for (;;) {
        if (*u16(msg) == 65535) break;
    }
}

void rx_objdata_follow (message *msg, objdata_element *el) {
    el->follow_oid = u32(msg);
    if (*el->follow_oid != 0xffffffff) {
        el->follow_xfres = u16(msg);
        el->follow_xfname = zstr(msg);
    }
}

void rx_objdata_homing (message *msg, objdata_element *el) {
    el->homing_oid = u32(msg);
    if (*el->homing_oid == 0xffffffff) {
    } else if (*el->homing_oid == 0xfffffffe) {
        el->homing_coord = coord(msg);
        el->homing_v = u16(msg);
    } else {
        el->homing_coord = coord(msg);
        el->homing_v = u16(msg);
    }
}

void rx_objdata_overlay (message *msg, objdata_element *el) {
    el->overlay_id = s32(msg);
    el->overlay_resid = u16(msg);
    if (*el->overlay_resid == 65535) {
    } else if ((*el->overlay_resid & 0x8000) != 0) {
        el->overlay_sdt = bytes(msg, *u8(msg));
    } else {
    }
}

void rx_objdata_auth (message *msg, objdata_element *el) {
    abort();
}

void rx_objdata_health (message *msg, objdata_element *el) {
    el->health = u8(msg);
}

void rx_objdata_buddy (message *msg, objdata_element *el) {
    el->buddy_name = zstr(msg);
    el->buddy_group = u8(msg);
    el->buddy_type = u8(msg);
}

void rx_objdata_cmppose (message *msg, objdata_element *el) {
    //TODO
    uint8_t pfl = *u8(msg);
    u8(msg);
    if ((pfl & 2) != 0) {
        for (;;) {
            uint16_t resid = *u16(msg);
            if (resid == 65535) break;
            if ((resid & 0x8000) != 0) {
                bytes(msg, *u8(msg));
            }
        }
    }
    if ((pfl & 4) != 0) {
        for (;;) {
            uint16_t resid = *u16(msg);
            if (resid == 65535) break;
            if ((resid & 0x8000) != 0) {
                bytes(msg, *u8(msg));
            }
        }
        u8(msg);
    }
}

void rx_objdata_cmpmod (message *msg, objdata_element *el) {
    //TODO
    for (;;) {
        uint16_t modif = *u16(msg);
        if (modif == 65535) break;
        for (;;) {
            if (*u16(msg) == 65535) break;
        }
    }
}

void rx_objdata_cmpequ (message *msg, objdata_element *el) {
    //TODO
    for (;;) {
        uint8_t h = *u8(msg);
        if (h == 255) break;
        zstr(msg);
        u16(msg);
        if (((h & 0x80) & 128) != 0) {
            s16(msg);
            s16(msg);
            s16(msg);
        }
    }
}

void rx_objdata_icon (message *msg, objdata_element *el) {
    el->icon_resid = u16(msg);
    if (*el->icon_resid != 65535) {
        el->icon_ifl = u8(msg);
    }
}

name_parse objdata_types[] = {
    [0 ]  = { .name =     "OD_REM", .parse = rx_objdata_rem     },
    [1 ]  = { .name =    "OD_MOVE", .parse = rx_objdata_move    },
    [2 ]  = { .name =     "OD_RES", .parse = rx_objdata_res     },
    [3 ]  = { .name =  "OD_LINBEG", .parse = rx_objdata_linbeg  },
    [4 ]  = { .name = "OD_LINSTEP", .parse = rx_objdata_linstep },
    [5 ]  = { .name =  "OD_SPEECH", .parse = rx_objdata_speech  },
    [6 ]  = { .name = "OD_COMPOSE", .parse = rx_objdata_compose },
    [7 ]  = { .name = "OD_DRAWOFF", .parse = rx_objdata_drawoff },
    [8 ]  = { .name =   "OD_LUMIN", .parse = rx_objdata_lumin   },
    [9 ]  = { .name =  "OD_AVATAR", .parse = rx_objdata_avatar  },
    [10]  = { .name =  "OD_FOLLOW", .parse = rx_objdata_follow  },
    [11]  = { .name =  "OD_HOMING", .parse = rx_objdata_homing  },
    [12]  = { .name = "OD_OVERLAY", .parse = rx_objdata_overlay },
    [13]  = { .name =    "OD_AUTH", .parse = rx_objdata_auth    },
    [14]  = { .name =  "OD_HEALTH", .parse = rx_objdata_health  },
    [15]  = { .name =   "OD_BUDDY", .parse = rx_objdata_buddy   },
    [16]  = { .name = "OD_CMPPOSE", .parse = rx_objdata_cmppose },
    [17]  = { .name =  "OD_CMPMOD", .parse = rx_objdata_cmpmod  },
    [18]  = { .name =  "OD_CMPEQU", .parse = rx_objdata_cmpequ  },
    [19]  = { .name =    "OD_ICON", .parse = rx_objdata_icon    }
};

void map_to_objdata_element (message *msg, objdata_element *el) {
    el->fl = u8(msg);
    el->id = s32(msg);
    el->frame = s32(msg);
    for (;;) {
        u_char type = *u8(msg);
        if (type == 255) break;
        objdata_types[type].parse(msg, el);
    }
}

void map_to_objdata (message *msg, salem_message *smsg) {
    while (msg->len > 0) {
        objdata_element *el = new_objdata_element(&smsg->objdata);
        map_to_objdata_element(msg, el);
    }
}

void print_objdata (salem_message *smsg) {
    u_int i;
    for (i=0; i<smsg->objdata.objs_len; ++i) {
        objdata_element *e = &smsg->objdata.objs[i];
        printf("    fl=%u id=%d frame=%d\n", *e->fl, *e->id, *e->frame);
        if (e->move_coord.x) printf("      move=[%d,%d] ia=%hu\n", *e->move_coord.x, *e->move_coord.y, *e->move_ia);
        if (e->linstep)      printf("      linstep=%d\n", *e->linstep);
        if (e->res_id)       printf("      resid=%hu sdt=[%s]\n", (uint16_t)((*e->res_id)&(~0x8000)), print_bseq(&e->res_sdt));
        if (e->health)       printf("      health=%u\n", *e->health);
    }
}

//////  OBJACK  //////////////////////////////////////////////////////////////////////

void map_to_objack (message *msg, salem_message *smsg) {}

//////  CLOSE  ///////////////////////////////////////////////////////////////////////

void map_to_close (message *msg, salem_message *smsg) {}





//////////////////////////////////////////////////////////////////////////////////////





char *rel_types[] = {
    [0 ] = "NEWWDG", 
    [1 ] = "WDGMSG", 
    [2 ] = "DSTWDG", 
    [3 ] = "MAPIV", 
    [4 ] = "GLOBLOB",
    [5 ] = "PAGINAE",
    [6 ] = "RESID",
    [7 ] = "PARTY",
    [8 ] = "SFX",
    [9 ] = "CATTR",
    [10] = "MUSIC",
    [11] = "TILES",
    [12] = "BUFF",
};

typedef struct {
    char *name;
    void (*map)(message *msg, salem_message *smsg);
    void (*print)(salem_message *smsg);
} name_map_print;

void print_rel     (salem_message *smsg) {
    printf("    seq=%hu\n", *smsg->rel.seq);
    u_int i;
    for (i=0; i<smsg->rel.rels_len; ++i) {
        printf("      type=%u %s\n", *smsg->rel.rels[i].type, rel_types[*smsg->rel.rels[i].type]);
    }
}
void print_ack     (salem_message *smsg) {}
void print_beat    (salem_message *smsg) {}
void print_mapreq  (salem_message *smsg) {}
void print_mapdata (salem_message *smsg) {}
void print_objack  (salem_message *smsg) {}
void print_close   (salem_message *smsg) {}

name_map_print msg_types[] = {
    [0] = { .name =    "SESS", .map = map_to_sess   , .print = print_sess    },
    [1] = { .name =     "REL", .map = map_to_rel    , .print = print_rel     },
    [2] = { .name =     "ACK", .map = map_to_ack    , .print = print_ack     },
    [3] = { .name =    "BEAT", .map = map_to_beat   , .print = print_beat    },
    [4] = { .name =  "MAPREQ", .map = map_to_mapreq , .print = print_mapreq  },
    [5] = { .name = "MAPDATA", .map = map_to_mapdata, .print = print_mapdata },
    [6] = { .name = "OBJDATA", .map = map_to_objdata, .print = print_objdata },
    [7] = { .name =  "OBJACK", .map = map_to_objack , .print = print_objack  },
    [8] = { .name =   "CLOSE", .map = map_to_close  , .print = print_close   }
};

void map (message *msg, salem_message *smsg) {
    smsg->from_server = msg->from_server;
    smsg->type = u8(msg);
    printf("  %s\n", msg_types[*smsg->type].name);
    msg_types[*smsg->type].map(msg, smsg);
}

void print (salem_message *smsg) {
    //printf("  %s\n", msg_types[*smsg->type].name);
    msg_types[*smsg->type].print(smsg);
}

void salem_parse (message *msg) {
    printf((msg->from_server)?"SERVER\n":"CLIENT\n");
    salem_message smsg;
    memset(&smsg, 0, sizeof(salem_message));
    map(msg, &smsg);
    print(&smsg);
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
