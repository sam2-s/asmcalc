ENTRY(_start)

SECTIONS
{
  . = 0x00010000;

  .text : {
    *(.text.start)
    *(.text .text.*)
  }

  .rodata : {
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
