# The DOS sample: 16-bit code in the style of a Borland Pascal program (our own code, for
# testing the harness). Every routine is FAR and uses the Pascal calling convention: arguments
# are pushed left to right, the callee removes them (`lretw $N`), results come back in AX
# (16-bit) or DX:AX (32-bit). DS holds the data segment (the NE "auto data" segment), as in a
# Pascal program. Data offsets are absolute in DS; make_exe.py builds the data segment to match.
#
# Pitfall: LLVM's Intel syntax assembles `retf 2` in .code16 as a 32-bit return (66 CA 02 00).
# This file uses AT&T syntax, where `lretw $2` gives the correct CA 02 00.
	.code16
	.set RandSeed, 0x0000   # longint: the random seed
	.set Total,    0x0004   # word: running sum kept by sum_draws
	.set Calls,    0x0006   # word: how many times rng_next ran


# program entry: load DS, draw once, exit to DOS with the result as the exit code
entry:
fix_entry_ds:
	movw $0xffff, %ax       # selector of the data segment (relocation)
	movw %ax, %ds
	pushw $100
	pushw %cs
	call rng_next
	movb $0x4c, %ah
	int $0x21

# procedure rng_seed(seed: longint)    pushed as high word, then low word
rng_seed:
	pushw %bp
	movw %sp, %bp
	movw 6(%bp), %ax
	movw %ax, RandSeed
	movw 8(%bp), %ax
	movw %ax, RandSeed+2
	movw $0, Calls
	popw %bp
	lretw $4

# function rng_next(range: word): word
#   RandSeed := RandSeed * $08088405 + 1;  result := (RandSeed shr 16) * range shr 16
rng_next:
	pushw %bp
	movw %sp, %bp
	movl RandSeed, %eax
	imull $0x08088405, %eax, %eax
	incl %eax
	movl %eax, RandSeed
	shrl $16, %eax
	mulw 6(%bp)
	movw %dx, %ax
	incw Calls
	popw %bp
	lretw $2

# function sum_draws(count, range: word): word    adds `count` draws to Total, returns it
sum_draws:
	pushw %bp
	movw %sp, %bp
	movw 8(%bp), %cx
1:	jcxz 2f
	pushw %cx
	pushw 6(%bp)
	pushw %cs               # a far call inside the same segment, as Borland emits it
	call rng_next
	addw %ax, Total
	popw %cx
	decw %cx
	jmp 1b
2:	movw Total, %ax
	popw %bp
	lretw $4

# function checksum(buf: pointer; len: word): word    sum of bytes, wrapping at 16 bits
#   a far pointer is pushed as segment, then offset
checksum:
	pushw %bp
	movw %sp, %bp
	lesw 8(%bp), %si
	movw 6(%bp), %cx
	xorw %ax, %ax
	xorw %bx, %bx
1:	jcxz 2f
	movb %es:(%si), %bl
	addw %bx, %ax
	incw %si
	decw %cx
	jmp 1b
2:	popw %bp
	lretw $6

# function get_data_seg: word    the data segment's selector, written by the loader
get_data_seg:
fix_dataseg:
	movw $0xffff, %ax
	lretw

# function ask_host: word    calls an imported function (KERNEL ordinal 3) and returns its AX
ask_host:
	.byte 0x9a              # call far ptr 0:0, filled in by the loader (relocation)
fix_import:
	.word 0xffff, 0
	lretw

# function dos_version: word    INT 21h AH=30h, returns AX (major in AL, minor in AH)
dos_version:
	movb $0x30, %ah
	int $0x21
	lretw
