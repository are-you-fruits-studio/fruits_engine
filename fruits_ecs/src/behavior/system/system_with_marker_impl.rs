use crate::*;

macro_rules! system_with_marker_impl {
    ($($P: ident),*) => {
        unsafe impl<F, $($P),*> SystemWithMarker<fn($($P),*)> for F
        where
            F: 'static + Send + Sync + Fn($($P),*) + for<'a> Fn($($P::Item<'a>),*),
            fn($($P),*): 'static,
            $($P: SystemParam),*
        {
            #[allow(redundant_semicolons)]
            fn fill_data_usage(&self, _usage: &mut DataUsage) {
                $($P::fill_data_usage(_usage));*;
            }
        
            unsafe fn execute<'e>(&self, _data: &SystemInput<'e>) {
                self(
                    // Safety. Managed by caller.
                    $(unsafe { get_param_or_panic::<F, $P>(_data) },)*
                );
            }

            fn into_system_generic(self) -> Box<dyn System> {
                Box::new(SystemWithMarkerAdapter::new(Box::new(self)))
            }

            fn system_name(&self) -> &'static str {
                std::any::type_name::<F>()
            }
        }
    };
}

fn panic_cannot_obtain_param<F, P>(msg: &'static str) -> ! {
    panic!(
        "System cannot obtain its parameters. System: {}. Parameter: {}. Message: {}",
        std::any::type_name::<F>(),
        std::any::type_name::<P>(),
        msg,
    )
}

unsafe fn get_param_or_panic<'e, F, P: SystemParam>(data: &'e SystemInput<'e>) -> P::Item<'e> {
    // Safety. Managed by caller.
    unsafe { P::new(data) }.unwrap_or_else(|m| panic_cannot_obtain_param::<F, P>(m))
}

system_with_marker_impl!();
system_with_marker_impl!(P0);
system_with_marker_impl!(P0, P1);
system_with_marker_impl!(P0, P1, P2);
system_with_marker_impl!(P0, P1, P2, P3);
system_with_marker_impl!(P0, P1, P2, P3, P4);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13);
system_with_marker_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14);