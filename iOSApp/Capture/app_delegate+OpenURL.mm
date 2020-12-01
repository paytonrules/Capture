//
//  AppDelegate+OpenUrl.mm
//  Capture
//
//  Created by Eric Smith on 11/19/20.
//  Copyright © 2020 Eric Smith. All rights reserved.
//
#import <objc/runtime.h>
#import <UIKit/UIKit.h>
#import "app_delegate.h"
#import <iostream>
#import "godot_capture.h"

@implementation AppDelegate (OpenURL)

- (BOOL)application:(UIApplication *)app openURL:(NSURL *)url options:(NSDictionary<UIApplicationOpenURLOptionsKey,id> *)options {

    logged_in([url.fragment cStringUsingEncoding:NSUTF8StringEncoding]);

    return TRUE;
}
@end
