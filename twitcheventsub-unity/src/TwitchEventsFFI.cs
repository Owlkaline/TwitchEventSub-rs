using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System;
using System.Collections;
using System.Collections.Generic;
using UnityEngine;
using System.Text;

public static class TwitchEventsFFI
{
    private static IntPtr twitch;

    [DllImport("librust_unity")]
    private static extern IntPtr get_event(IntPtr rngPtr);

    [DllImport("librust_unity")]
    private static extern IntPtr create_twitch_events(string subscriptions);//byte[] subscriptions);

    [DllImport("librust_unity")]
    private static extern void destroy_twitch_events(IntPtr twitch_events);

    [DllImport("librust_unity")]
    private static extern string extract_type(IntPtr event_data);

    [DllImport("librust_unity")]
    private static extern string extract_json(IntPtr event_data);

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static string event_type(IntPtr event_data)
    {
        return extract_type(event_data);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static string event_json(IntPtr event_data)
    {
        return extract_json(event_data);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static void Initialize(string subscriptions)
    {
        twitch = create_twitch_events(subscriptions);//Encoding.UTF8.GetBytes(subscriptions));
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static IntPtr GetEvents()
    {
        return get_event(twitch);
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static void Dispose()
    {
        destroy_twitch_events(twitch);
    }
}
