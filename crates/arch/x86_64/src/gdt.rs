use core::fmt;
use core::mem::size_of;

use crate::{PrivilegeLevel, VirtAddr};

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

/// A [*Segment Descriptor*](https://wiki.osdev.org/Global_Descriptor_Table#Segment_Descriptor)
/// within the [*Global Descriptor Table*](https://wiki.osdev.org/Global_Descriptor_Table).
///
/// # Representation
///
/// In 64-bit long mode, segment descriptor may be either one or two 64-bit words long.
/// Specifically, regular segment descriptors are one 64-bit word long, while system segments are
/// two 64-bit words long.
///
/// The `SIZE` generic parameter is used to distinguish between the possible sizes of descriptors.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SegmentDescriptor<const SIZE: usize>([u64; SIZE]);

impl<const SIZE: usize> SegmentDescriptor<SIZE> {
    /// The "null" segment descriptor.
    pub const NULL: Self = Self::from_raw([0x00; SIZE]);

    /// Creates a new [`SegmentDescriptor`] from the provided raw array.
    #[inline(always)]
    pub const fn from_raw(array: [u64; SIZE]) -> Self {
        Self(array)
    }

    /// Returns the inner 64-bit word array backing this [`SegmentDescriptor`].
    #[inline(always)]
    pub const fn to_raw(self) -> [u64; SIZE] {
        self.0
    }
}

impl SegmentDescriptor<1> {
    /// Creates a new 64-bit code [`SegmentDescriptor`].
    ///
    /// # Arguments
    ///
    /// - `present`: Whether the segment is present. Must be set for any valid segment.
    /// - `dpl`: The privilege level allowed to execute code within the segment.
    /// - `conforming`: Whether privilege levels bellow the `dpl` are also allowed to execute code
    /// within the segment.
    /// - `readable`: Whether the segment is readable.
    #[inline]
    pub const fn code(
        present: bool,
        dpl: PrivilegeLevel,
        conforming: bool,
        readable: bool,
    ) -> Self {
        let mut value = 0;

        value |= (present as u64) << 47;
        value |= (dpl as u64) << 45;
        value |= 1 << 44; // descriptor type
        value |= 1 << 43; // executable
        value |= (conforming as u64) << 42;
        value |= (readable as u64) << 41;
        value |= 1 << 53; // long mode code

        Self::from_raw([value])
    }

    /// Creates a new data [`SegmentDescriptor`].
    ///
    /// # Arguments
    ///
    /// - `present`: Whether the segment is present. Must be set for any valid segment.
    /// - `dpl`: The privilege level allowed to execute code within the segment.
    /// - `direction`: Whether the segment [grows downwards](https://wiki.osdev.org/Expand_Down)
    /// rather than upwards.
    /// - `writable`: Whether the segment is writable.
    #[inline]
    pub const fn data(present: bool, dpl: PrivilegeLevel, direction: bool, writable: bool) -> Self {
        let mut value = 0;

        value |= (present as u64) << 47;
        value |= (dpl as u64) << 45;
        value |= 1 << 44; // descriptor type
        value |= (direction as u64) << 42;
        value |= (writable as u64) << 41;

        Self::from_raw([value])
    }
}

impl SegmentDescriptor<2> {
    /// Creates a new 64-bit [**TSS**](https://wiki.osdev.org/Task_State_Segment) descriptor.
    ///
    /// # Arguments
    ///
    /// * `present`: Whether the descriptor is present. Must be set for any valid descriptor.
    ///
    /// * `dpl`: The privilege level of the segment.
    ///
    /// * `tss`: The virtual address of the *Task State Segment* structure.
    pub const fn tss(present: bool, dpl: PrivilegeLevel, tss: VirtAddr) -> Self {
        let mut high = 0;
        let mut low = 0;

        let limit = size_of::<Tss>() as u64 - 1;
        low |= limit & 0xFFFF;
        low |= (limit & 0xF0000) << 32;

        high |= (tss & 0xFFFFFFFF_00000000) >> 32;
        low |= (tss & 0xFF000000) << 32;
        low |= (tss & 0x00FFFFFF) << 16;
        low |= (present as u64) << 47;
        low |= (dpl as u64) << 45;
        low |= 0x9 << 40; // Available 64-bit TSS

        Self::from_raw([low, high])
    }

    /// Creates a new 64-bit [**LDT**](https://wiki.osdev.org/Local_Descriptor_Table) descriptor.
    ///
    /// # Arguments
    ///
    /// * `present`: Whether the descriptor is present. Must be set for any valid descriptor.
    ///
    /// * `dpl`: The privilege level of the segment.
    ///
    /// * `ldt`: The virtual address of the **LDT** structure.
    ///
    /// * `limit`: The size in bytes of the **LIDT**, minus one.
    ///
    /// # Panics
    ///
    /// In debug mode, this function panics if `limit` is larger than `0xFFFFF`.
    pub const fn ldt(present: bool, dpl: PrivilegeLevel, ldt: VirtAddr, limit: u64) -> Self {
        assert!(
            limit <= 0xFFFFF,
            "`SegmentDescriptor::ldt`: limit too large"
        );

        let mut high = 0;
        let mut low = 0;

        high |= (ldt & 0xFFFFFFFF_00000000) >> 32;
        low |= (ldt & 0xFF000000) << 32;
        low |= (ldt & 0x00FFFFFF) << 16;
        low |= limit & 0xFFFF;
        low |= (limit & 0xF0000) << 32;
        low |= (present as u64) << 47;
        low |= (dpl as u64) << 45;
        low |= 0x2 << 40; // LDT

        Self::from_raw([low, high])
    }
}

impl<const N: usize> fmt::Debug for SegmentDescriptor<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut l = f.debug_list();

        for &word in self.0.iter() {
            l.entry(&format_args!("{:#x}", word));
        }

        l.finish()
    }
}

/// A [Task State Segment](https://wiki.osdev.org/Task_State_Segment).
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tss {
    _reserved0: u32,
    rsp: [UnalignedVirtAddr; 3],
    _reserved1: [u32; 2],
    ist: [UnalignedVirtAddr; 7],
    _reserved2: [u32; 2],
    _reserved3: u16,
    iopb: u16,
}

impl Tss {
    /// Creates a new empty [`Tss`].
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            _reserved0: 0,
            rsp: [UnalignedVirtAddr::NULL; 3],
            _reserved1: [0; 2],
            ist: [UnalignedVirtAddr::NULL; 7],
            _reserved2: [0; 2],
            _reserved3: 0,
            iopb: 0,
        }
    }

    /// Sets a **Stack Pointer** within the *Interrupt Stack Table*.
    #[inline(always)]
    pub fn set_interrupt_stack(&mut self, index: IstIndex, addr: VirtAddr) {
        unsafe {
            self.ist.get_unchecked_mut(index as usize - 1).0 = addr;
        }
    }

    /// Sets the stack pointer that will be loaded when going from a higher privilege level to
    /// a lower one (`to_privilege`).
    ///
    /// # Safety
    ///
    /// `to_privilege` must not be [`PrivilegeLevel::Ring3`].
    ///
    /// # Panics
    ///
    /// In debug modes, this function panics if the provided `to_privilege` is
    /// `PrivilegeLevel::Ring3`.
    #[inline(always)]
    pub unsafe fn set_stack_pointer(&mut self, to_privilege: PrivilegeLevel, addr: VirtAddr) {
        debug_assert!(to_privilege != PrivilegeLevel::Ring3);

        unsafe {
            self.rsp.get_unchecked_mut(to_privilege as usize).0 = addr;
        }
    }
}

#[repr(C, packed(4))]
#[derive(Clone, Copy)]
struct UnalignedVirtAddr(VirtAddr);

impl UnalignedVirtAddr {
    /// The null pointer.
    pub const NULL: Self = Self(0);
}

impl fmt::Debug for UnalignedVirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;
        write!(f, "{:x}", val)
    }
}

impl fmt::Debug for Tss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("TaskStateSegment");

        s.field("rsp", &self.rsp);
        s.field("ist", &self.ist);
        s.field("iopb", &self.iopb);

        s.finish()
    }
}
