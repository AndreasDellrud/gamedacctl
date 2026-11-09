use gamedacctl::{
    ColorMarker, ColorShiftDuration, ColorShiftMode, LightingPlan, ProtocolError,
    profile::{Profile, ProfileLighting},
};

fn markers(values: &[&str]) -> Vec<ColorMarker> {
    values.iter().map(|v| v.parse().unwrap()).collect()
}

#[test]
fn duration_parsing_is_exact_and_bounded() {
    for (text, ticks) in [("1", 100), ("12.3", 1230), ("12.34", 1234), ("30.00", 3000)] {
        assert_eq!(
            text.parse::<ColorShiftDuration>().unwrap().centiseconds(),
            ticks
        );
    }
    for text in [
        "0.99", "30.01", "12.345", "12.", ".5", "NaN", "-1", "1e1", "65536", " 12",
    ] {
        assert!(text.parse::<ColorShiftDuration>().is_err(), "{text}");
    }
}

#[test]
fn layouts_and_unrepresentable_rates_fail_before_transport() {
    let duration = "12".parse().unwrap();
    for values in [
        vec![],
        vec!["FF0000@0"],
        vec!["FF0000@1", "0000FF@50"],
        vec!["FF0000@0", "0000FF@100"],
        vec!["FF0000@0", "0000FF@0"],
        vec!["FF0000@0", "0000FF@70", "00FF00@25"],
        vec!["FF0000@0", "0000FF@25", "00FF00@50", "FFFFFF@75"],
    ] {
        assert_eq!(
            LightingPlan::color_shift_markers(
                &markers(&values),
                duration,
                ColorShiftMode::Synchronized,
                false
            )
            .unwrap_err(),
            ProtocolError::InvalidMarkerLayout
        );
    }
    assert!(matches!(
        LightingPlan::color_shift_markers(
            &markers(&["FF0000@0", "0000FF@1"]),
            "1".parse().unwrap(),
            ColorShiftMode::Synchronized,
            false
        ),
        Err(ProtocolError::TransitionRateOutOfRange { .. })
    ));
    assert_eq!(
        LightingPlan::color_shift_markers(
            &markers(&["FF0000@0", "0000FF@50"]),
            duration,
            ColorShiftMode::Synchronized,
            true
        )
        .unwrap_err(),
        ProtocolError::ReverseRequiresReflected
    );
}

#[test]
fn reflected_swaps_earcups_without_wire_flags_and_preserves_aggregate_rounding() {
    let markers = markers(&["FF0000@0", "00FF00@25", "0000FF@70"]);
    let build = |mode, reverse| {
        LightingPlan::color_shift_markers(&markers, "12.34".parse().unwrap(), mode, reverse)
            .unwrap()
    };
    let sync = build(ColorShiftMode::Synchronized, false);
    let reflected = build(ColorShiftMode::Reflected, false);
    let reversed = build(ColorShiftMode::Reflected, true);
    for plan in [&sync, &reflected, &reversed] {
        assert_eq!(plan.zone_mask(), 3);
        assert_eq!(plan.outputs(), sync.outputs());
        for feature in plan.features() {
            assert_eq!(feature.bytes()[152], 0);
            assert_eq!(feature.bytes()[162], 0);
            assert_eq!(
                u16::from_le_bytes(feature.bytes()[160..162].try_into().unwrap()),
                1233
            );
        }
    }
    assert_eq!(sync.features()[1], reflected.features()[1]);
    assert_eq!(sync.features()[0], reversed.features()[0]);
    assert_ne!(sync.features()[0], reflected.features()[0]);
    assert_ne!(sync.features()[1], reversed.features()[1]);
}

#[test]
fn profiles_keep_legacy_json_and_round_trip_positioned_timing() {
    let legacy = r##"{"name":"Legacy","lighting":{"effect":"color-shift","colors":["#FF0000","#0000FF"],"seconds":12}}"##;
    let legacy: Profile = serde_json::from_str(legacy).unwrap();
    let before = legacy.plan().unwrap();
    let serialized = serde_json::to_value(&legacy).unwrap();
    assert_eq!(serialized["lighting"]["effect"], "color-shift");
    assert!(serialized["lighting"].get("markers").is_none());
    assert_eq!(
        serde_json::from_value::<Profile>(serialized)
            .unwrap()
            .plan()
            .unwrap(),
        before
    );
    let positioned = Profile {
        name: "Markers".into(),
        icon: None,
        microphone: None,
        lighting: ProfileLighting::ColorShiftMarkers {
            markers: markers(&["FF0000@0", "00FF00@25", "0000FF@70"]),
            centiseconds: 1234,
            gg122: false,
            mode: ColorShiftMode::Reflected,
            reverse: true,
        },
    };
    let after: Profile =
        serde_json::from_str(&serde_json::to_string(&positioned).unwrap()).unwrap();
    assert_eq!(after, positioned);
    assert_eq!(after.plan().unwrap(), positioned.plan().unwrap());
}

#[test]
fn gg_timing_clamps_short_segments_and_retains_legacy_profile_behavior() {
    let palette = markers(&["FF0000@0", "0000FF@1"]);
    let gg = LightingPlan::color_shift_markers_gg122(
        &palette,
        "1".parse().unwrap(),
        ColorShiftMode::Synchronized,
        false,
    )
    .unwrap();
    let report = gg.features()[0].bytes();
    assert_eq!(&report[12..15], &[(-123_i8) as u8, 0, 123]);
    assert_eq!(u16::from_le_bytes(report[16..18].try_into().unwrap()), 33);
    assert_eq!(
        u16::from_le_bytes(report[160..162].try_into().unwrap()),
        132
    );

    // Missing flag retains the previous positioned profile's packet timings.
    let old: Profile = serde_json::from_str(r##"{"name":"Existing marker profile","lighting":{"effect":"color-shift-markers","markers":[{"color":"#010000","percent":0},{"color":"#020000","percent":1}],"centiseconds":100,"mode":"synchronized","reverse":false}}"##).unwrap();
    let legacy = old.plan().unwrap();
    assert_eq!(legacy.features()[0].bytes()[16], 1);
    assert!(
        serde_json::to_value(&old).unwrap()["lighting"]
            .get("gg122")
            .is_none()
    );
    let mut opted_in = old.clone();
    if let ProfileLighting::ColorShiftMarkers { gg122, .. } = &mut opted_in.lighting {
        *gg122 = true;
    }
    assert_eq!(opted_in.plan().unwrap().features()[0].bytes()[16], 33);
    assert_eq!(old.plan().unwrap(), legacy);
}

#[test]
fn gg_timing_bounds_record_counts_and_builds_leading_holds() {
    let palette = (0..14)
        .map(|i| ColorMarker {
            color: gamedacctl::Color::new(i * 17, 255 - i * 17, i * 11),
            percent: 1 + i * 7,
        })
        .collect::<Vec<_>>();
    let gg = LightingPlan::color_shift_markers_gg122(
        &palette,
        "12.34".parse().unwrap(),
        ColorShiftMode::Reflected,
        false,
    )
    .unwrap();
    for feature in gg.features() {
        let b = feature.bytes();
        assert_eq!(b[158], 15);
        assert_eq!(&b[12..15], &[0, 0, 0]);
        assert_eq!(b[12 + 14 * 8 + 6], 0);
        assert!(!b[163..].iter().any(|v| *v != 0));
    }
    let fifteen = (0..15)
        .map(|i| ColorMarker {
            color: gamedacctl::Color::BLACK,
            percent: i * 6,
        })
        .collect::<Vec<_>>();
    assert!(
        LightingPlan::color_shift_markers_gg122(
            &fifteen,
            "12".parse().unwrap(),
            ColorShiftMode::Synchronized,
            false
        )
        .is_err()
    );
    let pulse = LightingPlan::breathe_markers(
        &markers(&["FF0000@10", "00FF00@35", "0000FF@80"]),
        "13.5".parse().unwrap(),
    )
    .unwrap();
    let b = pulse.features()[0].bytes();
    assert_eq!(b[158], 7);
    assert_eq!(&b[12..15], &[0, 0, 0]);
    assert_eq!(u16::from_le_bytes(b[16..18].try_into().unwrap()), 67);
    let ticks = (0..7)
        .map(|i| u16::from_le_bytes(b[16 + i * 8..18 + i * 8].try_into().unwrap()))
        .collect::<Vec<_>>();
    assert_eq!(ticks, vec![67, 162, 162, 297, 297, 135, 135]);
    assert_eq!(
        u16::from_le_bytes(b[160..162].try_into().unwrap()),
        ticks.iter().sum::<u16>()
    );
    assert!(
        LightingPlan::breathe_markers(
            &markers(&[
                "FF0000@0",
                "00FF00@20",
                "0000FF@40",
                "FFFFFF@60",
                "000000@80"
            ]),
            "12".parse().unwrap()
        )
        .is_err()
    );
}
