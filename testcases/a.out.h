/* The <a.out> header file describes the format of executable files. */

#ifndef _AOUT_H
#define _AOUT_H

struct exec {
	unsigned char a_magic[2];
	unsigned char a_flags;
	unsigned char a_cpu;
	unsigned char a_hdrlen;
	unsigned char a_unused;
	unsigned short a_version;
	long        a_text;
	long        a_data;
	long        a_bss;
	long        a_entry;
	long        a_total;
	long        a_syms;

	/* SHORT FORM ENDS HERE */
	long        a_trsize;
	long        a_drsize;
	long        a_tbase;
	long        a_dbase;
};
