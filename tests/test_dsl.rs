use dsl_example::meetup;

meetup!(
    RustBoulderDenver {
        location = "1601 Wewatta Street",
        meetings = [
            HackNSnack {
                attendees = ["Bob"],
            },
            Yells {
                topics = ["Proc Macros"],
                attendees = ["Alice"],
            },
        ]
    }
);

struct Response {
    profile: Option<Profile>,
}

struct Profile {
    name: Option<String>,
    display: Option<String>,
    real: Option<String>,
}

#[test]
fn test_dsl() {
    let meetup = RustBoulderDenver::default();

    assert_eq!(meetup.location(), "1601 Wewatta Street");
    assert_eq!(meetup.meetings()[0].attendees()[0], "Bob");
    if let RustBoulderDenverMeetings::Yells(y) = meetup.meetings()[1] {
        assert_eq!(y.topics()[0], "Proc Macros");
    }

    let response = Response {
        profile: None,
    };

    let t: Option<String> = response.profile.and_then(|p| {
        p.name.and(p.display).and(p.real)
    });

    let t = t.expect("This should always be populated");
}