MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the NRF52840 with Softdevices S140 7.2.0 */
  FLASH : ORIGIN = 0x00000000 + 156K, LENGTH = 1024K - 156K - 24K
  RAM : ORIGIN = 0x20000000 + 64K, LENGTH = 128K - 64K
}
