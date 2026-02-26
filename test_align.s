.section .text
.byte 0xAA      # Address 0x0
.align 4        # The ambiguous instruction
.byte 0xBB      # Where does this land?