//
//  NSObject+UIViewController_TestSwizzle.m
//  Capture
//
//  Created by Eric Smith on 11/19/20.
//  Copyright © 2020 GodotEngine. All rights reserved.
//
#import <objc/runtime.h>
#import <UIKit/UIKit.h>
#import "app_delegate.h"
#import "core/engine.h"

@implementation AppDelegate (OpenURL)

- (BOOL)application:(UIApplication *)app openURL:(NSURL *)url options:(NSDictionary<UIApplicationOpenURLOptionsKey,id> *)options {
    NSMutableDictionary *dic = [NSMutableDictionary new];
    for (NSString *segment in [url.fragment componentsSeparatedByString:@"&"]) {
        // key=value
        NSArray<NSString*> *pair = [segment componentsSeparatedByString:@"="];
        dic[pair[0]] = pair[1];
    }
    NSLog(@"openURL happened. All is well!");
    
    return TRUE;
}
@end
