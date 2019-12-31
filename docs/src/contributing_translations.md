If you'd like to add translations, take a look a look at the [English translation file](ui/src/translations/en.ts).

- Languages supported: English (`en`), Chinese (`zh`), Dutch (`nl`), Esperanto (`eo`), French (`fr`), Spanish (`es`), Swedish (`sv`), German (`de`), Russian (`ru`), Italian (`it`).

lang | done | missing
--- | --- | ---
de | 97% | avatar,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw 
eo | 84% | number_of_communities,preview,upload_image,avatar,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,theme,are_you_sure,yes,no 
es | 92% | avatar,archive_link,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw 
fr | 92% | avatar,archive_link,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw 
it | 93% | avatar,archive_link,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw 
nl | 86% | preview,upload_image,avatar,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,theme 
ru | 80% | cross_posts,cross_post,number_of_communities,preview,upload_image,avatar,formatting_help,view_source,sticky,unsticky,archive_link,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,recent_comments,theme,monero,by,to,transfer_community,transfer_site,are_you_sure,yes,no 
sv | 92% | avatar,archive_link,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw 
zh | 78% | cross_posts,cross_post,users,number_of_communities,preview,upload_image,avatar,formatting_help,view_source,sticky,unsticky,archive_link,settings,stickied,delete_account,delete_account_confirm,banned,creator,number_online,replies,mentions,forgot_password,reset_password_mail_sent,password_change,new_password,no_email_setup,language,browser_default,downvotes_disabled,enable_downvotes,open_registration,registration_closed,enable_nsfw,recent_comments,nsfw,show_nsfw,theme,monero,by,to,transfer_community,transfer_site,are_you_sure,yes,no 


If you'd like to update this report, run:

```bash 
cd ui
ts-node translation_report.ts > tmp # And replace the text above.
```