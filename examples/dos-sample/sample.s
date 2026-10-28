# The DOS sample: 16-bit code in the style of a Borland Pascal program (our own code, for
# testing GameFrame). Every routine is FAR and uses the Pascal calling convention: arguments
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

# function field_read: word    reads a field at [si+4]: offset 4 is also Total's address in
# DS, so a disassembler must not call it Total (nedis test)
field_read:
	movw 4(%si), %ax
	lretw

# function read_file: word    opens the file named at DS:0x0080 (DOS), reads up to 16 bytes into
# DS:0x0090, closes it, returns the count (or 0xFFFF when it cannot be opened). Tests dos_env.
	.set FileName, 0x0080
	.set FileBuf, 0x0090
	.set Ticks, 0x0008
read_file:
	movw $0x3d00, %ax
	movw $FileName, %dx
	int $0x21
	jc 1f
	movw %ax, %bx
	movb $0x3f, %ah
	movw $16, %cx
	movw $FileBuf, %dx
	int $0x21
	pushw %ax
	movb $0x3e, %ah
	int $0x21
	popw %ax
	lretw
1:	movw $0xffff, %ax
	lretw

# function wait_ticks: word    installs tick_handler on INT 8 (DOS AH=25h), waits until it has
# run 3 times (the timer interrupt), returns the count. Tests dos_env's timer.
wait_ticks:
	movw $0, Ticks
	pushw %ds
	pushw %cs
	popw %ds
	movw $0x2508, %ax
	movw $(tick_handler - entry), %dx   # an offset: entry is at 0 (no relocation)
	int $0x21
	popw %ds
	sti
1:	cmpw $3, Ticks
	jb 1b
	movw Ticks, %ax
	lretw
tick_handler:
	pushw %ds
	pushw %ax
	movw %cs:(fix_tick_ds - entry), %ax
	movw %ax, %ds
	incw Ticks
	popw %ax
	popw %ds
	iret
fix_tick_ds:
	.word 0xffff

# function get_key: word    INT 16h AH=00h: waits for a key, returns it (scan << 8 | ascii).
get_key:
	movb $0, %ah
	int $0x16
	lretw

# function write_file: word    opens the file named at DS:0x0080 read/write (mode 2, as Pascal's
# Reset does), writes 3 bytes from DS:0x0090, closes it; returns 0, or 0xFFFF (dos_env test:
# the game folder must not change).
write_file:
	movw $0x3d02, %ax
	movw $FileName, %dx
	int $0x21
	jc 1f
	movw %ax, %bx
	movb $0x40, %ah
	movw $3, %cx
	movw $FileBuf, %dx
	int $0x21
	movb $0x3e, %ah
	int $0x21
	xorw %ax, %ax
	lretw
1:	movw $0xffff, %ax
	lretw

# main_loop: a tiny game main loop for tools/emu/fragments.py (not called from entry). It gets
# two 64 KB blocks (the screen and a second heap block) and the VGA display from DPMI, keeps
# their selectors in DS, opens the file at FileName, then runs 8 frames and exits to DOS.
# Each frame has two fragments:
#   draw .. draw_even_end | draw_odd_end   writes the frame number into the screen and display
#                      (and the heap block on even frames), reads one byte of the file
#   event .. event_end  adds one to Event every 4th frame
	.set ScreenSeg, 0x00A0   # word: selector of the screen being drawn (a heap block)
	.set DisplaySeg, 0x00A2  # word: selector of the VGA display (A000h)
	.set HeapSeg, 0x00A4     # word: selector of a second heap block
	.set Frame, 0x00A6       # word: frame number, from 1
	.set Event, 0x00A8       # word: changed every 4th frame
	.set Handle, 0x00AA      # word: DOS handle of the open file
main_loop:
	movw $0x0100, %ax       # DPMI: DOS memory, BX paragraphs; DX = its selector
	movw $0x1000, %bx
	int $0x31
	movw %dx, ScreenSeg
	movw $0x0100, %ax
	movw $0x1000, %bx
	int $0x31
	movw %dx, HeapSeg
	movw $0x0002, %ax       # DPMI: real-mode segment A000h -> selector
	movw $0xa000, %bx
	int $0x31
	movw %ax, DisplaySeg
	movw $0x3d00, %ax
	movw $FileName, %dx
	int $0x21
	movw %ax, Handle
	movw $0, Frame
	movw $0, Event
frame_top:
	incw Frame
draw:
	movw Frame, %di
	movw %di, %ax
	movw ScreenSeg, %es
	movb %al, %es:(%di)
	movw DisplaySeg, %es
	movb %al, %es:(%di)
	movb $0x3f, %ah         # read 1 byte of the file into FileBuf
	movw Handle, %bx
	movw $1, %cx
	movw $FileBuf, %dx
	int $0x21
	testw $1, Frame
	jnz draw_odd_end
	movw HeapSeg, %es
	movw Frame, %ax
	movb %al, %es:(%di)
draw_even_end:
	jmp event
draw_odd_end:
	nop
event:
	testw $3, Frame
	jnz event_end
	incw Event
event_end:
	cmpw $8, Frame
	jb frame_top
	movw $0x4c00, %ax
	int $0x21

# function wait_keys(count: word): word    installs key_handler on INT 9 (DOS AH=25h), waits
# until it has seen `count` scan codes, returns how many it has seen. key_handler does what a
# game's keyboard handler does: reads the scan code from port 0x60, marks the key down or up in
# KeyDown (0x0100 + scan), logs it in KeyLog, and acknowledges the PIC (EOI to port 0x20).
# Tests dos_env's keyboard (IRQ 1).
	.set KeyCount, 0x00AC   # word: scan codes seen
	.set KeyLog, 0x00B0     # 16 bytes: the scan codes, in order (wraps)
	.set KeyDown, 0x0100    # 128 bytes: 1 while the key is held
wait_keys:
	pushw %bp
	movw %sp, %bp
	pushw %ds
	pushw %cs
	popw %ds
	movw $0x2509, %ax
	movw $(key_handler - entry), %dx
	int $0x21
	popw %ds
	sti
	movw 6(%bp), %cx
1:	cmpw %cx, KeyCount
	jb 1b
	movw KeyCount, %ax
	popw %bp
	lretw $2
key_handler:
	pushw %ds
	pushw %ax
	pushw %bx
	movw %cs:(fix_tick_ds - entry), %ax   # the data segment (tick_handler's relocated word)
	movw %ax, %ds
	inb $0x60, %al
	movw KeyCount, %bx
	andw $15, %bx
	movb %al, KeyLog(%bx)
	incw KeyCount
	movb %al, %bl
	andw $0x7f, %bx
	notb %al
	shrb $7, %al            # 1 for a make code, 0 for a break code
	movb %al, KeyDown(%bx)
	movb $0x20, %al
	outb %al, $0x20
	popw %bx
	popw %ax
	popw %ds
	iret

# Three routines nothing calls, at the end so no other offset moves: Ghidra finds none of them.
# ghidra-run.sh's decompile by address makes a function only where one may start: after a far
# return (retf CB, retf N CA nn nn), not after an iret (tests/run.sh --ghidra).
after_iret:
	movw $7, %ax
	lretw
after_retf:
	movw $8, %ax
	lretw $2
after_retf_n:
	movw $9, %ax
	lretw

# Two more that Ghidra misses, for exportcov's 16-bit starts (tests/run.sh): far_caller opens
# with push bp / mov bp, sp right after a far return, and far_only is reached only by its far
# call. The call holds the offset; the selector is 0xffff until the loader fills it from an NE
# relocation (make_exe.py). Neither runs.
far_caller:
	pushw %bp
	movw %sp, %bp
	.byte 0x9a              # call far far_only
	.word far_only - entry
fix_far_only:
	.word 0xffff
	popw %bp
	lretw
far_only:
	movw $10, %ax
	lretw
