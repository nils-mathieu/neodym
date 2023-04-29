use core::fmt;

use crate::PrivilegeLevel;

/// A descriptor table. Can either be the *Global Descriptor Table* or the *Local Descriptor Table*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptorTable {
    /// The *Global Descriptor Table*.
    Gdt,
    /// The current *Local Descriptor Table*.
    Ldt,
}

/// A [segment selector](https://wiki.osdev.org/Segment_Selector).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    /// Creates a new [`SegmentSelector`] from its inner raw value.
    #[inline(always)]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Returns the inner 16-bit value of this [`SegmentSelector`].
    #[inline(always)]
    pub const fn to_raw(self) -> u16 {
        self.0
    }

    /// Creates a new [`SegmentSelector`].
    ///
    /// # Arguments
    ///
    /// * `index`: The index of the **GDT** or **LDT** entry referenced by this segment selector.
    /// Note that this is not a *byte index* (i.e. index `1` is the second entry).
    /// * `ti`: Specifies which descriptor table to use with this selector. Either the **GDT** or
    /// the current **LDT**.
    /// * `rpl`: The *requested privilege level* of the selector.
    ///
    /// # Panics
    ///
    /// In debug builds, this function panics if `index * 8` overflows an `u16`.
    pub const fn new(index: u16, ti: DescriptorTable, rpl: PrivilegeLevel) -> Self {
        let mut value = 0;

        value |= index << 3;
        value |= (ti as u16) << 2;
        value |= rpl as u16;

        Self::from_raw(value)
    }

    /// Returns the index of the **GDT** or **LDT** entry referenced by this segment selector.
    #[inline(always)]
    pub const fn index(self) -> u16 {
        self.0 >> 3
    }

    /// Returns the table descriptor to use with this [`SegmentSelector`].
    #[inline(always)]
    pub const fn table(self) -> DescriptorTable {
        if (self.0 >> 2) & 1 == 0 {
            DescriptorTable::Gdt
        } else {
            DescriptorTable::Ldt
        }
    }

    /// Returns the requested privilege level of this [`SegmentSelector`].
    #[inline(always)]
    pub const fn requested_privilege_level(self) -> PrivilegeLevel {
        unsafe { PrivilegeLevel::from_raw_unchecked(self.0 as u8 & 0b11) }
    }
}

impl fmt::Debug for SegmentSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SegmentSelector")
            .field("index", &self.index())
            .field("table", &self.table())
            .field("rpl", &self.requested_privilege_level())
            .finish()
    }
}

/// An index within the
/// [Interrupt Stack Table](https://wiki.osdev.org/Task_State_Segment#Long_Mode).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_docs)]
pub enum IstIndex {
    One = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl IstIndex {
    /// Creates a new [`IstIndex`] from the provided raw value.
    ///
    /// # Safety
    ///
    /// The provided `raw` value must be non-zero, and less or equal to 7.
    ///
    /// # Panics
    ///
    /// In debug builds, this function panics if `raw` is not a valid [`IstIndex`].
    #[inline(always)]
    pub const unsafe fn from_raw_unchecked(raw: u8) -> Self {
        unsafe { core::mem::transmute(raw) }
    }
}

impl fmt::Debug for IstIndex {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = *self as u8;
        fmt::Debug::fmt(&idx, f)
    }
}
