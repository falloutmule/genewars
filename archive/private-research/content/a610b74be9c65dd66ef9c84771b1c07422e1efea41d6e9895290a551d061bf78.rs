//! Resource selection from CD 0x15B90 and the mission-view transition.
//! This is a bounded path/state plan, not a mission simulation or completion rule.

/// Safe port admission. The original uses signed byte arithmetic without these
/// bounds; the supplied campaign assets support this restricted range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignPlanError {
    UnsupportedProgress(i8),
    UnsupportedPriorSystem(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignResourcePlan {
    pub progress: i8,
    pub local_planet: u16,
    /// Retained on planets two through six, even when inconsistent with progress.
    pub effective_system: u8,
    pub header: String,
    pub mission: String,
    pub texture_atlas: String,
    pub campaign_background: String,
    pub campaign_palette: String,
    pub system_movie: Option<String>,
    pub transition_background: String,
    pub transition_palette: String,
}

impl CampaignResourcePlan {
    /// Select paths without reading files or applying an original-game command.
    ///
    /// CD 0x15B90 only assigns the stored system byte when remainder+1 is one.
    /// Textures and transition assets use that retained byte; the campaign
    /// background independently uses progress/6. Do not collapse those sources.
    /// Paths use the original relative naming forms with portable separators.
    pub fn new(progress: i8, prior_system: u8) -> Result<Self, CampaignPlanError> {
        if !(0..24).contains(&progress) {
            return Err(CampaignPlanError::UnsupportedProgress(progress));
        }
        if prior_system > 3 {
            return Err(CampaignPlanError::UnsupportedPriorSystem(prior_system));
        }
        let quotient = progress as u8 / 6;
        let local_planet = u16::from(progress as u8 % 6) + 1;
        let effective_system = if local_planet == 1 { quotient } else { prior_system };
        let number = progress as u8 + 1;
        Ok(Self {
            progress,
            local_planet,
            effective_system,
            header: format!("LEVELS/hdrco{number}.dat"),
            mission: format!("levels/conq{number:03}.dat"),
            texture_atlas: format!("data/pl{}.dat", effective_system + 1),
            campaign_background: format!("data/levscr0{}.dat", quotient + 1),
            campaign_palette: format!("data/levscr0{}.pal", quotient + 1),
            system_movie: (local_planet == 1).then(|| format!("anims/system{}.smk", effective_system + 1)),
            transition_background: format!("data/toPlnt{effective_system}.dat"),
            transition_palette: format!("data/toPlnt{effective_system}.pal"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_campaign_selects_traced_first_mission_resources() {
        let plan = CampaignResourcePlan::new(0, 3).unwrap();
        assert_eq!(plan.local_planet, 1);
        assert_eq!(plan.effective_system, 0);
        assert_eq!(plan.header, "LEVELS/hdrco1.dat");
        assert_eq!(plan.mission, "levels/conq001.dat");
        assert_eq!(plan.texture_atlas, "data/pl1.dat");
        assert_eq!(plan.campaign_background, "data/levscr01.dat");
        assert_eq!(plan.campaign_palette, "data/levscr01.pal");
        assert_eq!(plan.system_movie.as_deref(), Some("anims/system1.smk"));
        assert_eq!(plan.transition_background, "data/toPlnt0.dat");
        assert_eq!(plan.transition_palette, "data/toPlnt0.pal");
    }

    #[test]
    fn nonboundary_progress_retains_system_and_separates_background_selection() {
        let plan = CampaignResourcePlan::new(7, 3).unwrap();
        assert_eq!(plan.local_planet, 2);
        assert_eq!(plan.effective_system, 3);
        assert_eq!(plan.texture_atlas, "data/pl4.dat");
        assert_eq!(plan.campaign_background, "data/levscr02.dat");
        assert_eq!(plan.campaign_palette, "data/levscr02.pal");
        assert_eq!(plan.transition_background, "data/toPlnt3.dat");
        assert_eq!(plan.system_movie, None);
        assert_eq!(plan.mission, "levels/conq008.dat");
    }

    #[test]
    fn sequential_campaign_updates_system_at_each_six_planet_boundary() {
        let mut stored = 0;
        for progress in 0..24 {
            let plan = CampaignResourcePlan::new(progress, stored).unwrap();
            stored = plan.effective_system;
            assert_eq!(stored, progress as u8 / 6);
            assert_eq!(plan.local_planet, (progress as u16 % 6) + 1);
            assert_eq!(plan.system_movie.is_some(), progress % 6 == 0);
        }
        let last = CampaignResourcePlan::new(23, stored).unwrap();
        assert_eq!(last.header, "LEVELS/hdrco24.dat");
        assert_eq!(last.mission, "levels/conq024.dat");
        assert_eq!(last.local_planet, 6);
    }

    #[test]
    fn unsupported_state_is_rejected_without_wrapping_into_resource_names() {
        for progress in i8::MIN..=i8::MAX {
            assert_eq!(CampaignResourcePlan::new(progress, 0).is_ok(), (0..24).contains(&progress));
        }
        for prior in 4..=u8::MAX {
            assert_eq!(CampaignResourcePlan::new(0, prior), Err(CampaignPlanError::UnsupportedPriorSystem(prior)));
        }
    }
}
