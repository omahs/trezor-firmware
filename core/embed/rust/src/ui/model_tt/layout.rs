use core::convert::{TryFrom, TryInto};

use crate::{
    error::Error,
    micropython::{buffer::Buffer, obj::Obj},
    ui::{
        component::{Child, FormattedText},
        display,
        layout::obj::LayoutObj,
    },
    util,
};

use super::{
    component::{Button, ButtonMsg, Dialog, DialogMsg},
    theme,
};

impl<T> TryFrom<DialogMsg<T, ButtonMsg, ButtonMsg>> for Obj
where
    Obj: TryFrom<T>,
    Error: From<<Obj as TryFrom<T>>::Error>,
{
    type Error = Error;

    fn try_from(val: DialogMsg<T, ButtonMsg, ButtonMsg>) -> Result<Self, Self::Error> {
        match val {
            DialogMsg::Content(c) => Ok(c.try_into()?),
            DialogMsg::Left(ButtonMsg::Clicked) => 1.try_into(),
            DialogMsg::Right(ButtonMsg::Clicked) => 2.try_into(),
            _ => Ok(Obj::const_none()),
        }
    }
}

#[no_mangle]
extern "C" fn ui_layout_new_example(param: Obj) -> Obj {
    let block = move || {
        let param: Buffer = param.try_into()?;
        let layout = LayoutObj::new(Child::new(Dialog::new(
            display::screen(),
            |area| {
                FormattedText::new::<theme::TTDefaultText>(area, param)
                    .with(b"some", "a few")
                    .with(b"param", "xx")
            },
            |area| Button::with_text(area, b"Left"),
            |area| Button::with_text(area, b"Right"),
        )))?;
        Ok(layout.into())
    };
    unsafe { util::try_or_raise(block) }
}

#[cfg(test)]
mod tests {
    use crate::trace::{Trace, Tracer};

    use super::*;

    impl Tracer for Vec<u8> {
        fn int(&mut self, i: i64) {
            self.string(&i.to_string());
        }

        fn bytes(&mut self, b: &[u8]) {
            self.extend(b)
        }

        fn string(&mut self, s: &str) {
            self.extend(s.as_bytes())
        }

        fn symbol(&mut self, name: &str) {
            self.extend(name.as_bytes())
        }

        fn open(&mut self, name: &str) {
            self.extend(b"<");
            self.extend(name.as_bytes());
            self.extend(b" ");
        }

        fn field(&mut self, name: &str, value: &dyn Trace) {
            self.extend(name.as_bytes());
            self.extend(b":");
            value.trace(self);
            self.extend(b" ");
        }

        fn close(&mut self) {
            self.extend(b">")
        }
    }

    fn trace(val: &impl Trace) -> String {
        let mut t = Vec::new();
        val.trace(&mut t);
        String::from_utf8(t).unwrap()
    }

    #[test]
    fn trace_example_layout() {
        let layout = Child::new(Dialog::new(
            display::screen(),
            |area| {
                FormattedText::new::<theme::TTDefaultText>(
                    area,
                    "Testing text layout, with some text, and some more text. And {param}",
                )
                .with(b"param", b"parameters!")
            },
            |area| Button::with_text(area, b"Left"),
            |area| Button::with_text(area, b"Right"),
        ));
        assert_eq!(
            trace(&layout),
            r#"<Dialog content:<Text content:Testing text layout, with
some text, and some more
text. And parameters! > left:<Button text:Left > right:<Button text:Right > >"#
        )
    }
}