use godot::prelude::*;
use twitcheventsub::prelude::{BadgeData, BadgeVersion, EmoteStaticImages, SetOfBadges};

use crate::GEmoteStaticImages;

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GBadgeVersion {
  #[var]
  pub id: GString,
  #[var]
  pub images: Gd<GEmoteStaticImages>,
  #[var]
  pub title: GString,
  #[var]
  pub description: GString,
  #[var]
  pub click_action: Variant,
  #[var]
  pub click_url: Variant, //Option<GString>,
}

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GBadgeData {
  #[var]
  pub set_id: GString,
  #[var]
  pub versions: Array<Gd<GBadgeVersion>>,
}

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GSetOfBadges {
  #[var]
  pub badges: Array<Gd<GBadgeData>>,
}

impl From<BadgeVersion> for GBadgeVersion {
  fn from(value: BadgeVersion) -> Self {
    GBadgeVersion {
      id: value.id.to_godot(),
      images: Gd::from_object(GEmoteStaticImages::from(value.images)),
      title: value.title.to_godot(),
      description: value.description.to_godot(),
      click_action: if let Some(action) = value.click_action {
        Variant::from(action)
      } else {
        Variant::nil()
      },
      click_url: if let Some(url) = value.click_url {
        Variant::from(url)
      } else {
        Variant::nil()
      },
    }
  }
}

impl Into<BadgeVersion> for GBadgeVersion {
  fn into(self) -> BadgeVersion {
    BadgeVersion {
      id: self.id.into(),
      images: EmoteStaticImages {
        url_1x: self.images.bind().get_url_1x().into(),
        url_2x: self.images.bind().get_url_2x().into(),
        url_4x: self.images.bind().get_url_4x().into(),
      },
      title: self.title.into(),
      description: self.description.into(),
      click_action: if self.click_action.is_nil() {
        None
      } else {
        Some(self.click_action.to())
      },
      click_url: if self.click_url.is_nil() {
        None
      } else {
        Some(self.click_url.to())
      },
    }
  }
}

impl From<BadgeData> for GBadgeData {
  fn from(value: BadgeData) -> Self {
    GBadgeData {
      set_id: value.set_id.to_godot(),
      versions: value
        .versions
        .into_iter()
        .map(|v| Gd::from_object(GBadgeVersion::from(v)))
        .collect::<Array<_>>(),
    }
  }
}

impl Into<BadgeData> for GBadgeData {
  fn into(self) -> BadgeData {
    BadgeData {
      set_id: self.set_id.into(),
      versions: self
        .versions
        .iter_shared()
        .map(|v| (*v.bind()).clone().into())
        .collect::<Vec<_>>(),
    }
  }
}

impl From<SetOfBadges> for GSetOfBadges {
  fn from(value: SetOfBadges) -> Self {
    GSetOfBadges {
      badges: value
        .data
        .into_iter()
        .map(|b| Gd::from_object(GBadgeData::from(b)))
        .collect::<Array<_>>(),
    }
  }
}

impl Into<SetOfBadges> for GSetOfBadges {
  fn into(self) -> SetOfBadges {
    SetOfBadges {
      data: self
        .badges
        .iter_shared()
        .map(|b| (*b.bind()).clone().into())
        .collect::<Vec<_>>(),
    }
  }
}
