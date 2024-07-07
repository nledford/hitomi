use nutype::nutype;

#[nutype(
    derive(
        Clone,
        Default,
        Debug,
        Deserialize,
        Display,
        PartialEq,
        Serialize,
        AsRef,
        Deref
    ),
    default = "New Profile/Playlist",
    sanitize(trim),
    validate(not_empty, len_char_max = 500)
)]
pub struct Title(String);

#[cfg(test)]
mod profile_title_tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_valid_title() {
        let valid_title = "Valid Title";
        let title = Title::try_new(valid_title).unwrap();
        assert_eq!(valid_title, title.into_inner())
    }

    #[test]
    fn test_invalid_title_blank() {
        let expected = Err(TitleError::NotEmptyViolated);
        let result = Title::try_new("");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_title_too_long() {
        let expected = Err(TitleError::LenCharMaxViolated);
        // 1,111 characters long, generated from `/dev/urandom`
        let invalid_title = r#"
            &*Tr4^WR>QBISvgGp#GcJ;dZHABne[-7;ilaM/k'+d+'npe.+c3G*(VN87k)I2H'iEPatEU}2MHhd@;:(83E
            <lZul2cCa,BxCosJ?pZ`[J4TZ>/WW)Ry[R88R"CS4tV>o%+Ut:-R@;W/qJZ6V^C?m(plOB|gx?m[*ypc#uM3
            ssC@_jLQAV[`DHYqLy*pzJz'QN$qwR,Ukh93N&)#s#et5hwp(h<7wd1UM}l1hoy:(Ym[Q/m,A3cGXh0W@A[]
            nn!*6.FI/oXA*OHlR-5q)UKbfD*37px,74G/cVKiEl^Ricim}_q#B@m}vo:=YpqK8QdV:T,NoV'kwfQz.i0K
            B6qCU?'o?}#A:x;EKhsgi^H>gZFY9?q<>)(A@En7-9sG^kv5Jr'6r)zTsG+3[It5a^E8+hSxmp97`E+^pUeS
            E%~z)<QJR1y6^t%gH)e`C_z+)<&6I}m|:5DZ,HwM$B]a3Y<;E$19f7{.lRL<4QOA+^ip@12UYPo_f74G<(Z>
            ,Mq]}}cTa!%81.xUxhc}VmGib0t)=rPD'W`KVUpLQYneiB{+<ij3qxt"fBwGHQpk7nI3zHyM!4vm<s,*lz|+
            lD/UchDFN<mNNqUR}]jwqp'e;$j}Wc>;$OHw_wsB_jH,v@*pApWB+8o2GV6O_>*v|SN'J}9yjaE(ZdbJ]V%W
            aM>!p(*vy(<l([Wcm%$h$)q~]Y:i{Te7*Do--DMdK`5R^LSw]`GFkyUhmk-['$f-:M@j}p[&(1fX}}!W'CSg
            tfY!f4uAD2o,;*4By9duM37eCp/iP(yq-MzT'66l*ABT9<&J-,/ifgnd|}Hs8dvUrvzZ=w0qP&g;DueJj?2v
            ,!]=O"&PaGDfz[4r0ND$!Yg05y-F5,,h.Z8"n8VQX==^RYsrG|,(<<8H^@qz{|QE7_l6,3'BU5&-V200LC@w
            /@ynQHgt{Ea?*,H+/~^8y]7NXv](!/;{cB/o""|^bp<Y1a6BXMTlOa+aea70w;vYw}5OUyE3*oM`Ct!0jWR!
            .0_'ge/!!V0b-+Nu(WYv=pW[|.o-L9dan"vGJ2>8{f@|jS68o$#QVT:^O7:d:af-h9~A.8xjyzY0PO;e0ya@
            @Xpex;3)w@<2PdW[<r}

        "#;
        let result = Title::try_new(invalid_title);
        assert_eq!(expected, result)
    }
}
