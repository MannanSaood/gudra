use crate::{Error, Result};

pub(crate) const TILE: usize = 16;
const MAX_BLOCKS: usize = 65_535;

/// Validated interior `[height, width]` for a compact row-major `f32` grid.
///
/// The corresponding source has physical extent `[height + 2, width + 2]`.
/// A one-cell halo is fixed; tiny and non-tile-aligned interiors are supported.
/// Construction performs no allocation or GPU work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape2D {
    // Compact, checked metadata keeps ShapeMismatch's two shapes small without
    // heap allocation. All values originate as usize and must round-trip.
    interior: [u32; 2],
    haloed: [u32; 2],
    interior_len: u32,
    haloed_len: u32,
    interior_bytes: usize,
    haloed_bytes: usize,
    grid: [u16; 3],
}

impl Shape2D {
    /// Validates the interior dimensions, counts, bytes, strides, and launch grid.
    ///
    /// # Errors
    ///
    /// Rejects zero axes first, then arithmetic overflow, byte counts above
    /// `isize::MAX`, axes/strides/total counts above `i32::MAX`, and finally more
    /// than 65,535 blocks on either axis of the internal 16x16 tile grid.
    pub fn new(height: usize, width: usize) -> Result<Self> {
        for (axis, extent) in [("height", height), ("width", width)] {
            if extent == 0 {
                return Err(Error::EmptyInterior { axis });
            }
        }
        let halo_height = height.checked_add(2).ok_or(Error::ShapeOverflow {
            operation: "height + 2",
        })?;
        let halo_width = width.checked_add(2).ok_or(Error::ShapeOverflow {
            operation: "width + 2",
        })?;
        let interior_len = product(height, width, "height * width")?;
        let haloed_len = product(halo_height, halo_width, "halo height * halo width")?;
        let interior_bytes = product(interior_len, size_of::<f32>(), "interior bytes")?;
        let haloed_bytes = product(haloed_len, size_of::<f32>(), "haloed bytes")?;
        limit("interior bytes", interior_bytes, isize::MAX as usize)?;
        limit("haloed bytes", haloed_bytes, isize::MAX as usize)?;
        // Widths also bound contiguous row strides. Total counts bound the
        // backend's i32 stride products and flat upload length conversion.
        for (quantity, actual) in [
            ("height", height),
            ("width", width),
            ("halo height", halo_height),
            ("halo width", halo_width),
            ("interior elements", interior_len),
            ("haloed elements", haloed_len),
        ] {
            limit(quantity, actual, i32::MAX as usize)?;
        }
        let blocks_y = height.div_ceil(TILE);
        let blocks_x = width.div_ceil(TILE);
        limit("height blocks", blocks_y, MAX_BLOCKS)?;
        limit("width blocks", blocks_x, MAX_BLOCKS)?;
        let grid = [block_count(blocks_y)?, block_count(blocks_x)?, 1];
        Ok(Self {
            interior: [element_count(height)?, element_count(width)?],
            haloed: [element_count(halo_height)?, element_count(halo_width)?],
            interior_len: element_count(interior_len)?,
            haloed_len: element_count(haloed_len)?,
            interior_bytes,
            haloed_bytes,
            grid,
        })
    }

    /// Number of interior rows.
    #[must_use]
    pub fn height(self) -> usize {
        self.interior[0] as usize
    }

    /// Number of interior columns.
    #[must_use]
    pub fn width(self) -> usize {
        self.interior[1] as usize
    }

    /// Required element count for the RHS and output.
    #[must_use]
    pub fn interior_len(self) -> usize {
        self.interior_len as usize
    }

    /// Required element count for the source, including every halo cell.
    #[must_use]
    pub fn haloed_len(self) -> usize {
        self.haloed_len as usize
    }

    #[cfg(feature = "gpu")]
    pub(crate) fn interior(self) -> [usize; 2] {
        self.interior.map(|axis| axis as usize)
    }

    #[cfg(feature = "gpu")]
    pub(crate) fn haloed(self) -> [usize; 2] {
        self.haloed.map(|axis| axis as usize)
    }
}

fn product(left: usize, right: usize, operation: &'static str) -> Result<usize> {
    left.checked_mul(right)
        .ok_or(Error::ShapeOverflow { operation })
}

fn limit(quantity: &'static str, actual: usize, maximum: usize) -> Result<()> {
    if actual > maximum {
        return Err(Error::ShapeLimit {
            quantity,
            actual,
            maximum,
        });
    }
    Ok(())
}

fn element_count(count: usize) -> Result<u32> {
    u32::try_from(count).map_err(|_| Error::ShapeLimit {
        quantity: "metadata elements",
        actual: count,
        maximum: i32::MAX as usize,
    })
}

fn block_count(blocks: usize) -> Result<u16> {
    u16::try_from(blocks).map_err(|_| Error::ShapeLimit {
        quantity: "launch blocks",
        actual: blocks,
        maximum: MAX_BLOCKS,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangular_geometry_and_ceil_grid() {
        let shape = Shape2D::new(17, 19).unwrap();
        assert_eq!(shape.interior, [17, 19]);
        assert_eq!(shape.haloed, [19, 21]);
        assert_eq!(shape.interior_len(), 323);
        assert_eq!(shape.haloed_len(), 399);
        assert_eq!(shape.interior_bytes, 1_292);
        assert_eq!(shape.haloed_bytes, 1_596);
        assert_eq!(shape.grid, [2, 2, 1]);
    }

    #[test]
    fn rejects_zero_before_overflow() {
        assert!(matches!(
            Shape2D::new(0, usize::MAX),
            Err(Error::EmptyInterior { axis: "height" })
        ));
        assert!(matches!(
            Shape2D::new(usize::MAX, 0),
            Err(Error::EmptyInterior { axis: "width" })
        ));
        assert!(matches!(
            Shape2D::new(usize::MAX, 1),
            Err(Error::ShapeOverflow { .. })
        ));
        assert!(matches!(
            Shape2D::new(usize::MAX / 2, 3),
            Err(Error::ShapeOverflow { .. })
        ));
    }

    #[test]
    fn bounds_total_elements_and_each_grid_axis() {
        assert!(matches!(
            Shape2D::new(50_000, 50_000),
            Err(Error::ShapeLimit { .. })
        ));
        let last = MAX_BLOCKS * TILE;
        assert_eq!(Shape2D::new(last, 1).unwrap().grid, [65_535, 1, 1]);
        assert_eq!(Shape2D::new(1, last).unwrap().grid, [1, 65_535, 1]);
        assert!(matches!(
            Shape2D::new(last + 1, 1),
            Err(Error::ShapeLimit {
                quantity: "height blocks",
                ..
            })
        ));
        assert!(matches!(
            Shape2D::new(1, last + 1),
            Err(Error::ShapeLimit {
                quantity: "width blocks",
                ..
            })
        ));
    }
}
