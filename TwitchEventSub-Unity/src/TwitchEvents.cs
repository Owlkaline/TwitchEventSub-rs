using System.Collections;


using System.Collections.Generic;
using UnityEngine;
using UnityEngine.Events;

using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

[System.Serializable]
public struct CheerMote
{
    public string prefix;
    public uint bits;
    public uint tier;
}

[System.Serializable]
public struct Emote
{
    public string id;
    public string emote_set_id;
    public string owner_id;
    public string format;
}

[System.Serializable]
public struct Mention
{
    public string user_id;
    public string user_login;
    public string user_name;
}

[System.Serializable]
public struct Fragments
{
    public string kind;
    public string text;
    public CheerMote cheermote;
    public Emote emote;
    public Mention mention;
}

[System.Serializable]
public struct Message
{
    public string text;
    public Fragments[] fragments;
}

[System.Serializable]
public struct Badge
{
    public string set_id;
    public string id;
    public string info;
}

[System.Serializable]
public struct Cheer
{
    public uint bits;
}

[System.Serializable]
public struct Reply
{
    public string thread_user_id;
    public string thread_user_name;
    public string thread_user_login;
    public string parent_user_id;
    public string parent_user_name;
    public string parent_user_login;
    public string parent_message_id;
    public string parent_message_body;
    public string thread_message_id;
}

[System.Serializable]
public class UMessageData
{
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public string chatter_user_id;
    public string chatter_user_name;
    public string chatter_user_login;
    public string message_id;
    public Message message;
    public string color;
    public string message_type;
    public Cheer cheer;
    public Reply reply;
    public string channel_points_custom_reward_id;
    public string channel_points_animation_id;
}

[System.Serializable]
public class UAdBreakBegin
{
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public string requester_user_id;
    public string requester_user_name;
    public string requester_user_login;
    public int duration_seconds;
    public string start_at;
    public bool is_automatic;
}

[System.Serializable]
public class UCheer
{
    public string user_id;
    public string user_login;
    public string user_name;
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public bool is_anonymous;
    public string message;
    public int bits;
}


[System.Serializable]
public class URewardEmote
{
    public string id;
    public int begin;
    public int end;
}

[System.Serializable]
public class UFollow
{
    public string user_id;
    public string user_login;
    public string user_name;
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public string followed_at;
}

[System.Serializable]
public class URaid
{
    public string to_broadcaster_user_id;
    public string to_broadcaster_user_name;
    public string to_broadcaster_user_login;
    public string from_broadcaster_user_id;
    public string from_broadcaster_user_name;
    public string from_broadcaster_user_login;
    public int viewers;
}

[System.Serializable]
public class UReward
{
    public string id;
    public string title;
    public string prompt;
    public int cost;
}

[System.Serializable]
public class UCustomRewardRedeem
{
    public string id;
    public string user_id;
    public string user_login;
    public string user_name;
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public string user_input;
    public string status;
    public UReward reward;
    public string redeemed_at;
}

[System.Serializable]
public class URewardMessage
{
    public string text;
    public URewardEmote[] emotes;
}

[System.Serializable]
public class UNewSubscription
{
    public string user_id;
    public string user_login;
    public string user_name;
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public string tier;
    public bool is_gift;
}

[System.Serializable]
public class UResubscription
{
    public string user_id;
    public string user_login;
    public string user_name;
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public URewardMessage message;
    public int cumulative_months;
    public int streak_months;
    public int duration_months;
}

[System.Serializable]
public class UGift
{
    public string user_id;
    public string user_login;
    public string user_name;
    public string broadcaster_user_id;
    public string broadcaster_user_name;
    public string broadcaster_user_login;
    public int total;
    public string tier;
    public int cumulative_total;
    public bool is_anonymous;
}

[System.Serializable]
public class ChatMessageEvent : UnityEvent<UMessageData> { }
[System.Serializable]
public class ChatMessagePowerupGigantifiedEmoteEvent : UnityEvent<UMessageData> { }
[System.Serializable]
public class ChatMessagePowerupMessageEffectEvent : UnityEvent<UMessageData> { }
[System.Serializable]
public class CustomPointRewardRedeemEvent : UnityEvent<UCustomRewardRedeem> { }
[System.Serializable]
public class AdBreakStartEvent : UnityEvent<UAdBreakBegin> { }
[System.Serializable]
public class RaidEvent : UnityEvent<URaid> { }
[System.Serializable]
public class FollowEvent : UnityEvent<UFollow> { }
[System.Serializable]
public class NewSubscriptionEvent : UnityEvent<UNewSubscription> { }
[System.Serializable]
public class SubscriptionGiftEvent : UnityEvent<UGift> { }
[System.Serializable]
public class ResubscriptionEvent : UnityEvent<UResubscription> { }
[System.Serializable]
public class CheerEvent : UnityEvent<UCheer> { }

[System.Serializable]
public class Subscritpions
{
    public bool user_update = false;
    public bool follow = true;
    public bool raid = true;
    public bool update = false;
    public bool new_subscription = true;
    public bool subscription_end = false;
    public bool gift_subscription = true;
    public bool resubscription = true;
    public bool cheer = true;
    public bool points_custom_reward_redeem = true;
    public bool points_auto_reward_redeem = true;
    public bool poll_begin = false;
    public bool poll_progress = false;
    public bool poll_end = false;
    public bool prediction_begin = false;
    public bool prediction_progress = false;
    public bool prediction_lock = false;
    public bool prediction_end = false;
    public bool goal_begin = false;
    public bool goal_progress = false;
    public bool goal_end = false;
    public bool hype_train_begin = false;
    public bool hype_train_progress = false;
    public bool hype_train_end = false;
    public bool shoutout_create = false;
    public bool shoutout_receive = false;
    public bool ban_timeout_user = false;
    public bool delete_message = false;
    public bool ad_break_begin = true;
    public bool chat_message = true;
}

public class TwitchEvents : MonoBehaviour
{
    public ChatMessageEvent chat_message_event;
    public ChatMessagePowerupGigantifiedEmoteEvent chat_message_powerup_gigantified_emote_event;
    public ChatMessagePowerupMessageEffectEvent chat_message_powerup_message_effect_event;
    public CustomPointRewardRedeemEvent custom_point_reward_redeem_event;
    public AdBreakStartEvent ad_break_start_event;
    public RaidEvent raid_event;
    public FollowEvent follow_event;
    public NewSubscriptionEvent new_subscription_event;
    public SubscriptionGiftEvent subscription_gift_event;
    public ResubscriptionEvent resubscription_event;
    public CheerEvent cheer_event;

    public Subscritpions subscriptions;

    void Awake()
    {
    }

    // Start is called before the first frame update
    void Start()
    {
        string subs = JsonUtility.ToJson(subscriptions);
        //Debug.Log(subs);
        TwitchEventsFFI.Initialize(subs);
    }

    // Update is called once per frame
    void Update()
    {
        var event_data = TwitchEventsFFI.GetEvents();
        var kind = TwitchEventsFFI.event_type(event_data);
        var json = TwitchEventsFFI.event_json(event_data);
        if (kind == "")
        {
            return;
        }

        if (kind == "chat_message")
        {
            var new_message = JsonUtility.FromJson<UMessageData>(json);
            chat_message_event.Invoke(new_message);
        }
        else
        if (kind == "chat_message_powerup_gigantified_emote")
        {
            var new_message = JsonUtility.FromJson<UMessageData>(json);
            chat_message_powerup_gigantified_emote_event.Invoke(new_message);
        }
        else
        if (kind == "chat_message_powerup_message_effect")
        {
            var new_message = JsonUtility.FromJson<UMessageData>(json);
            chat_message_powerup_message_effect_event.Invoke(new_message);
        }
        else
        if (kind == "custom_point_reward_redeem")
        {
            var new_message = JsonUtility.FromJson<UCustomRewardRedeem>(json);
            custom_point_reward_redeem_event.Invoke(new_message);
        }
        else
        if (kind == "ad_break_start")
        {
            var new_message = JsonUtility.FromJson<UAdBreakBegin>(json);
            ad_break_start_event.Invoke(new_message);
        }
        else
        if (kind == "raid")
        {
            var new_message = JsonUtility.FromJson<URaid>(json);
            raid_event.Invoke(new_message);
        }
        else
        if (kind == "follow")
        {
            var new_message = JsonUtility.FromJson<UFollow>(json);
            follow_event.Invoke(new_message);
        }
        else
        if (kind == "new_subscritption")
        {
            var new_message = JsonUtility.FromJson<UNewSubscription>(json);
            new_subscription_event.Invoke(new_message);
        }
        else
        if (kind == "subscription_gift")
        {
            var new_message = JsonUtility.FromJson<UGift>(json);
            subscription_gift_event.Invoke(new_message);
        }
        else
        if (kind == "resubscription")
        {
            var new_message = JsonUtility.FromJson<UResubscription>(json);
            resubscription_event.Invoke(new_message);
        }
        else
        if (kind == "cheer")
        {
            var new_message = JsonUtility.FromJson<UCheer>(json);
            cheer_event.Invoke(new_message);
        }
    }

    void onDestroy()
    {
        TwitchEventsFFI.Dispose();
    }
}
