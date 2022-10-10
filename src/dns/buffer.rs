type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

pub struct BytePacketBuffer {
    pub buf: [u8; 512],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> BytePacketBuffer {
        BytePacketBuffer { 
            buf: [0; 512],
            pos: 0,
        }
    }

    // Return current buffer position
    pub fn pos(&self) -> usize {
        self.pos
    }

    // Increment the buffer position n times
    pub fn step(&mut self, steps: usize) ->Result<()> {
        self.pos += steps;

        Ok(())
    }

    // Seek the buffer position
    pub fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;

        Ok(())
    }

    // Get byte, seeking buffer position after we read it
    pub fn read(&mut self) -> Result<u8> {
        if self.pos >= 512 {
            return Err("End of buffer".into());
        }
        let res = self.buf[self.pos];
        self.pos += 1;

        Ok(res)
    }

    // Get byte without seeking buffer
    pub fn get(&mut self, pos: usize) -> Result<u8> {
        if pos >= 512 {
            // this would need eDNS
            return Err("End of buffer".into());
        }
        Ok(self.buf[pos])
    }

    // Get a range of bytes
    pub fn get_range(&mut self, start: usize, len: usize) -> Result<&[u8]> {
        if start + len >= 512 {
            return Err("End of buffer".into());
        }
        Ok(&self.buf[start..start + len as usize])
    }

    // read 2 bytes, seeking appropriately
    pub fn read_u16(&mut self) -> Result<u16> {
        let res = ((self.read()? as u16) << 8) | (self.read()? as u16);
        Ok(res)
    }

    // read 4 bytes, seeking appropriately
    pub fn read_u32(&mut self) -> Result<u32> {
        let res = ((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | ((self.read()? as u32) << 0);

        Ok(res)
    }

    pub fn read_qname(&mut self, outstr: &mut String) -> Result<()> {
        let mut pos = self.pos();

        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;

        let mut delim = "";
        loop {
            if jumps_performed > max_jumps {
                return Err(
                    format!("Limit of {} jumps exceeded", max_jumps).into()
                );
            }

            let len = self.get(pos)?;
            // When the two most signficant bits are set, this means we have a jump
            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos + 2)?;
                }

                let next_byte = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | next_byte;
                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;

                continue;
            }
            // Default case - no jump
            else {
                pos += 1;
                // Labels are zero-terminated. A zero signifies the end of the domain name.
                if len == 0 {
                    break;
                }

                outstr.push_str(delim);
                let str_buffer = self.get_range(pos, len as usize)?;
                outstr.push_str(
                    &String::from_utf8_lossy(str_buffer)
                    .to_lowercase()
                );

                delim = ".";
                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(())
    }

    pub fn write(&mut self, val: u8) -> Result<()> {
        if self.pos >= 512 {
            return Err("End of buffer".into());
        }
        self.buf[self.pos] = val;
        self.pos += 1;

        Ok(())
    }

    pub fn write_u8(&mut self, val: u8) -> Result<()> {
        self.write(val)?;

        Ok(())
    }

    pub fn write_u16(&mut self, val: u16) -> Result<()> {
        self.write((val >> 8) as u8)?;
        self.write((val & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_u32(&mut self, val: u32) -> Result<()> {
        self.write(((val >> 24) & 0xFF) as u8)?;
        self.write(((val >> 16) & 0xFF) as u8)?;
        self.write(((val >> 8) & 0xFF) as u8)?;
        self.write(((val >> 0) & 0xFF) as u8)?;

        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split(".") {
            let len = label.len();
            if len > 0x3f {
                return Err(
                    "Single label exceeds 63 characters".into()
                );
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }

        }

        self.write_u8(0)?;
        Ok(())
    }

    pub fn set(&mut self, pos: usize, val: u8) -> Result<()> {
        self.buf[pos] = val;

        Ok(())
    }

    pub fn set_u16(&mut self, pos: usize, val: u16) -> Result<()> {
        self.set(pos, (val >> 8) as u8)?;
        self.set(pos + 1, (val & 0xFF) as u8)?;

        Ok(())
    }
}
