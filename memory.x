MEMORY
{
  # setup for standard Clue S140 v6 - use 0x27000 for S140 v7
  FLASH (rx)     : ORIGIN = 0x26000, LENGTH = 0xED000 - 0x26000

  /* SRAM required by Softdevice depend on
   * - Attribute Table Size (Number of Services and Characteristics)
   * - Vendor UUID count
   * - Max ATT MTU
   * - Concurrent connection peripheral + central + secure links
   * - Event Len, HVN queue, Write CMD queue
   */ 
  RAM (rwx) :  ORIGIN = 0x20006000, LENGTH = 0x20040000 - 0x20006000
}

