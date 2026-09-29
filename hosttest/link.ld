ENTRY(_start)

SECTIONS
{
  . = 0x00010000;

  .text : ALIGN(4) {
    *(.text.start)
    *(.text .text.*)
  }

  .rodata : ALIGN(4) {
    *(.rodata .rodata.*)
  }

  . = ALIGN(0x1000);

  .data : {
    *(.data .data.*)
  }

  .bss (NOLOAD) : ALIGN(4) {
    *(.bss .bss.*)
    *(COMMON)
  }

  /DISCARD/ : {
    *(.comment)
    *(.note.*)
    *(.eh_frame)
    *(.interp)
    *(.dynamic)
    *(.dynsym)
    *(.dynstr)
    *(.hash)
    *(.gnu.hash)
  }
}
